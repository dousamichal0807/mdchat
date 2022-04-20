/*
 * Copyright (c) 2022  Michal Douša.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::{client_list, message_list};
use crate::decrypt;
use crate::encrypt;
use crate::global_config;
use crate::log;
use crate::message_queue;
use crate::user_list;

use mdchat_common::command::c2s;
use mdchat_common::command::s2c;
use mdchat_common::login::LoginRequest;

use mdlog::LogLevel;

use mdswp::MdswpStream;

use std::io;
use std::io::Read;
use std::io::Write;
use std::mem::size_of;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Client {
    socket_addr: SocketAddr,
    stream: RwLock<MdswpStream>,
    nickname: RwLock<Option<String>>,
}

impl Client {
    /// Creates a new [`Client`] instance from given [`MdswpStream`].
    ///
    /// > **Note!**
    /// >
    /// > There should be no [`MdswpStream`] socket clones. If reading or writing to
    /// > the [`MdswpStream`] socket is done outside this struct, it leads to
    /// > unpredictable behavior.
    pub fn new(stream: MdswpStream) -> Arc<Client> {
        Arc::new(Self {
            socket_addr: stream.peer_addr().unwrap(),
            stream: RwLock::new(stream),
            nickname: RwLock::new(Option::None)
        })
    }

    /// Returns client's socket address
    pub fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }

    /// Returns what nickname is client logged into.
    ///
    /// # Return value
    ///
    ///  -  [`Option::Some`] with the nickname if the user is logged in.
    ///  -  [`Option::None`] if the user is not logged in
    pub fn nickname(&self) -> Option<String> {
        self.nickname.write().unwrap().clone()
    }

    /// This is a method that should be run is a seperate thread each time after
    /// a new [`Client`] instance is constructed.
    pub fn client_thread(&self) {
        while !self.is_err() {
            // Next command:
            let command = match self.recv_command() {
                Result::Ok(Option::Some(command)) => command,
                Result::Ok(Option::None) => {
                    let _ = self.stream.write().unwrap().finish_write();
                    return
                }
                Result::Err(err) => {
                    self.error(err.to_string());
                    return
                },
            };
            // Process command:
            match command {
                c2s::Command::Login(request) => self.on_login(request),
                c2s::Command::SendMessage(text) => self.on_message(text),
            };
        }
        // Remove connection when error occurred:
        client_list::remove_connection(&self.socket_addr);
    }

    /// Returns if the underlying [`MdswpStream`] has errored. See
    /// [`MdswpStream::is_err`] for more information.
    pub fn is_err(&self) -> bool {
        self.stream.read().unwrap().is_err()
    }

    /// Sends given [`s2c::Command`] to the client. Since [`ClientInfo`] uses
    /// synchronization internally, this method can be called concurrently in
    /// different threads.
    pub fn send_command(&self, command: s2c::Command) -> io::Result<()> {
        let json = serde_json::to_string(&command).unwrap();
        let encrypted = encrypt(&json.into_bytes());
        if encrypted.len() > u32::MAX as usize {
            return Result::Err(io::Error::new(io::ErrorKind::InvalidInput, "Data too large"));
        }
        let mut stream = self.stream.write().unwrap();
        stream.write_all(&(encrypted.len() as u32).to_be_bytes())?;
        stream.write_all(&encrypted)?;
        stream.flush()?;
        Result::Ok(())
    }

    /// This method should be used to signal an error. This method will
    /// automatically inform client about error that happened and will close the
    /// connection.
    pub fn error(&self, err: String) {
        // Send response back to client:
        let command = s2c::Command::Error(err.to_string());
        let _ = self.send_command(command);
        let mut stream = self.stream.write().unwrap();
        let _ = stream.finish_write();
        let _ = stream.reset();
    }

    #[doc(hidden)]
    fn recv_command(&self) -> io::Result<Option<c2s::Command>> {
        // Lock stream
        let mut stream = self.stream.read().unwrap().try_clone()?;
        // Read exactly four bytes which will denote next message length:
        let mut buffer = [0; size_of::<u32>()];
        let read_bytes = stream.read(&mut buffer)?;
        // If no byte has been read, it is OK:
        if read_bytes == 0 { return Result::Ok(Option::None) }
        // If we have not read all four bytes, it is an error:
        if read_bytes < size_of::<u32>() {
            return Result::Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "End of stream was not expected"
            ))
        }
        // The data length:
        let data_len = u32::from_be_bytes(buffer) as usize;
        // Prepare a buffer with length of `data_len` bytes and read exactly
        // `data_len` bytes:
        let mut buffer = vec![0; data_len];
        stream.read_exact(&mut buffer[0..data_len])?;
        // Decrypt
        let decrypted = decrypt(&buffer[0..data_len]);
        // Convert to `String`:
        let string = String::from_utf8(decrypted)
            .map_err(|err| format!("Received invalid data: {}", err))
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        // Deserialize:
        serde_json::from_str(&string)
            .map_err(|err| format!("Received invalid data: {}", err))
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    #[doc(hidden)]
    fn on_login(&self, request: LoginRequest) {
        let is_registering = request.is_registering();
        let nickname = request.nickname();
        let password = request.password();
        // Check nickname policy:
        if !global_config().is_allowed_nickname(nickname) {
            let log_message = format!("Tried to used banned nickname `{}`", nickname);
            let client_message = format!("`{}` is not an allowed nickname due to regulations.", nickname);
            self.error(client_message);
            log(LogLevel::Warning, &log_message);
            return
        }
        // Get if nickname is already registered:
        let is_present = user_list::exists(&nickname);
        // Do something based on if client is trying to register and given nickname
        // already exists
        match (is_registering, is_present) {
            (true, true) => self.register_error_already_exists(&nickname),
            (true, false) => self.register(nickname.clone(), password.clone()),
            (false, true) => self.login(nickname.clone(), password.clone()),
            (false, false) => self.login_error_not_existing(&nickname)
        }
    }

    #[doc(hidden)]
    fn register_error_already_exists(&self, nickname: &str) {
        let log_message = format!("Tried to register already existing nickname: `{}`", nickname);
        let client_message = format!("`{}` is already existing user account", nickname);
        self.error(client_message);
        log(LogLevel::Info, &log_message);
    }

    #[doc(hidden)]
    fn register(&self, nickname: String, password: String) {
        user_list::add_user(nickname.clone(), password.clone());
        *self.nickname.write().unwrap() = Option::Some(nickname.clone());
        let log_message = format!("Successfully registered and logged in as `{}`", nickname);
        log(LogLevel::Info, &log_message);
    }

    #[doc(hidden)]
    fn login(&self, nickname: String, password: String) {
        // If client tried to log in with wrong password, kick it:
        if !user_list::verify_password(&nickname, password.clone()) {
            self.error("Invalid password".to_string());
            log(LogLevel::Warning, "Tried to log in as {} with invalid password")
        }
        // Send LoginSuccess
        if let Result::Err(err) = self.send_command(s2c::Command::LoginSuccess) {
            self.error(err.to_string());
            return
        }
        // Update nickname
        *self.nickname.write().unwrap() = Option::Some(nickname.clone());
        // Log successful login
        let message = format!("Logged in as `{}`", nickname);
        log(LogLevel::Info, &message);
        // Send messages that were sent when the user was not connected,
        // only if last send message ID is present:
        let last_msg_id = user_list::get_last_sent_msg_id(&nickname);
        if let Option::Some(last_msg_id) = last_msg_id {
            message_list::for_messages_newer_than(last_msg_id, |_, message| {
                match self.send_command(s2c::Command::MessageRecv(message.clone())) {
                    Result::Ok(()) => {},
                    Result::Err(err) => self.error(err.to_string())
                }
            });
        }
    }

    #[doc(hidden)]
    fn login_error_not_existing(&self, nickname: &str) {
        let log_message = format!("Tried to log into a non-existing account: `{}`", nickname);
        let client_message = format!("User with nickname `{}` does not exist", nickname);
        self.error(client_message);
        log(LogLevel::Warning, &log_message);
    }

    #[doc(hidden)]
    fn on_message(&self, text: String) {
        match self.nickname() {
            Option::Some(nickname) => message_queue::push(nickname, text),
            Option::None => {
                let message = "Tried to send a message while not logged in";
                self.error(message.to_string());
                log(LogLevel::Warning, message);
            },
        }
    }
}