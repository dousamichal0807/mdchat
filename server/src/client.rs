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

use chrono::Utc;

use crate::client_list;
use crate::message_list;
use crate::message_queue;
use crate::user_list;

use mdswp::MdswpStream;

use std::error::Error;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::thread::JoinHandle;

pub struct ClientInfo {
    pub socket_addr: SocketAddr,
    pub connection: MdswpStream,
    pub nickname: Option<String>,
    pub thread_handle: JoinHandle<io::Result<()>>
}

pub fn client_thread(mut stream: MdswpStream, peer_addr: SocketAddr) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream.try_clone()?);

    // TODO: log(LogKind::Info, format!("{} has sucessfully connected", addr))

    let login_bytes = match listen_for(&mut reader) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => todo!(),
        Err(err) => {
            let error = io::Error::new(
                err.kind(),
                format!("{}: {:?}", peer_addr, err.kind())
            );
            handle_err(&peer_addr, &error).unwrap();
            return Err(err);
        }
    };
    todo!("let login_bytes = MESSAGE_CRYPT.decrypt(login_bytes);");
    let login_string = match String::from_utf8(login_bytes) {
        Ok(string) => string,
        Err(_) => {
            let error = io::Error::new(
                io::ErrorKind::InvalidInput,
                "sent non-UTF-8 data"
            );
            handle_err(&peer_addr, &error).unwrap();
            return Err(error);
        }
    };
    let login_split: Vec<&str> = login_string.splitn(2, "\n").into_iter().collect();
    if login_split.len() != 2 {
        let err = io::Error::new(
            io::ErrorKind::InvalidInput,
            "was expected to send nickname and password to login"
        );
        handle_err(&peer_addr, &err).unwrap();
        return Err(err);
    }
    let nickname = login_split[0];
    let password = login_split[1];

    match client_list::login(&peer_addr, nickname.to_string(), password.to_string()) {
        Ok(_) => {},
        Err(err) => { handle_err(&peer_addr, &err).unwrap(); return Err(err); }
    }

    // TODO: log(LogKind::Info, format!("{} logged in as {}", addr, nickname));

    let mut state = Ok(());

    match user_list::get_last_sent_msg_id(nickname).unwrap() {
        Some(msg_id) => message_list::for_messages_newer_than(msg_id, |_, message| {
            if matches!(state, Ok(())) {
                state = send_info(&mut writer, message.to_string());
            }
        }),
        None => {}
    };

    match state {
        Ok(()) => {},
        Err(err) => { handle_err(&peer_addr, &err).unwrap(); return Err(err); }
    }

    loop {
        let message = match listen_for(&mut reader) {
            Err(err) => return match err.kind() {
                io::ErrorKind::UnexpectedEof => Ok(()),
                _other => {
                    handle_err(&peer_addr, &err).unwrap();
                    Err(err)
                }
            },
            Ok(None) => {
                // TODO: log(LogKind::Info, format!("{} disconnected", addr));
                return Ok(());
            }
            Ok(Some(byte_vec)) => {
                let decrypted_bytes = todo!(); // MESSAGE_CRYPT.decrypt(byte_vec);
                match String::from_utf8(decrypted_bytes) {
                    Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
                    Ok(text) => text,
                }
            }
        };

        if message.starts_with(":") {
            let split: Vec<&str> = message[1..].trim().splitn(2, " ").collect();
            let command_name = split[0];
            let command_arg = split.get(1);

            if command_name.is_empty() {
                send_info(&mut writer, "Expected command after colon (`:`)".to_string())?;
            } else if command_name == "q" || command_name == "quit" {
                match command_arg {
                    Some(_) => send_info(&mut writer, "Quit command should have no arguments".to_string())?,
                    None => {
                        stream.finish_write()?;
                        client_list::remove_connection(&peer_addr)?;
                        return Ok(())
                    }
                }
            } else {
                send_info(&mut writer, format!("Invalid command: {}", command_name))?;
            }
        } else {
            let trim = message.trim();
            message_queue::push(&peer_addr, Utc::now(), trim.to_string()).unwrap();
        }
    }
}

pub fn listen_for<R>(reader: &mut R) -> io::Result<Option<Vec<u8>>>
    where
        R: Read,
{
    let mut iter = reader.by_ref().take(2);
    let mut buffer = [0; 2];
    match iter.read(&mut buffer)? {
        0 => return Ok(None),
        2 => {}, // continue
        _ => return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "end of transmission was not expected"
        ))
    }
    let data_len = u16::from_be_bytes(buffer) as usize;

    let mut buffer = Vec::with_capacity(data_len);
    for _ in 0..data_len {
        buffer.push(0);
    }

    reader.read_exact(&mut buffer[0..data_len])?;
    Ok(Some(buffer))
}

pub fn handle_err<E>(addr: &SocketAddr, error: &E) -> io::Result<()>
    where
        E: Error,
{
    // TODO: log(LogKind::Error, format!("{} {}: {:?}", addr, config, config));
    let encrypted: Vec<u8> = todo!(); // MESSAGE_CRYPT.encrypt(config.to_string().into_bytes());
    if encrypted.len() > u16::MAX.into() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Data too large"));
    }
    let client_info = client_list::remove_connection(&addr)?;
    let mut stream = client_info.connection;
    match {
        stream.write(&(encrypted.len() as u16).to_be_bytes())?;
        stream.write(&encrypted)?;
        stream.flush()
    } {
        Ok(()) => {},
        Err(err) => todo!() // log(LogKind::Warning, format!("{} {}", addr, err))
    };
    Ok(())
}

pub fn send_info<W>(writer: &mut W, info: String) -> io::Result<()>
    where
        W: Write,
{
    let bytes = info.into_bytes();
    let encrypted_bytes: Vec<u8> = todo!(); //MESSAGE_CRYPT.encrypt(bytes);
    let len = encrypted_bytes.len();

    if len > u16::MAX.into() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Data too long"))
    }

    let len = len as u16;
    writer.write(&len.to_be_bytes())?;
    writer.write(&encrypted_bytes)?;
    writer.flush()?;
    Ok(())
}