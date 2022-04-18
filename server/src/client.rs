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

use chrono::DateTime;
use chrono::Utc;

use crate::client_list;
use crate::decrypt;
use crate::encrypt;
use crate::log;
use crate::message_list;
use crate::message_queue;
use crate::user_list;

use mdchat_common::command::c2s;
use mdchat_common::command::s2c;

use mdlog::LogLevel;

use mdswp::MdswpStream;

use std::error::Error;
use std::fmt::Display;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::mem::size_of;
use std::net::SocketAddr;
use std::thread::JoinHandle;

pub struct ClientInfo {
    pub socket_addr: SocketAddr,
    pub stream: MdswpStream,
    pub nickname: Option<String>,
    pub thread_handle: JoinHandle<()>
}

pub fn client_thread(mut stream: MdswpStream, peer_addr: SocketAddr) {
    while !stream.is_err() {
        // Next command:
        let command = match recv_command(&mut stream) {
            Result::Ok(Option::Some(command)) => command,
            Result::Ok(Option::None) => {
                let _ = stream.finish_write();
                return
            }
            Result::Err(err) => {
                handle_err(&peer_addr, err);
                return
            },
        };
        // Process command:
        match command {
            c2s::Command::Fetch(date_time) => __process_fetch(peer_addr, date_time),
            c2s::Command::Login { is_registering, nickname, password } =>
                client_list::login(&peer_addr, is_registering, nickname, password),
            c2s::Command::SendMessage(text) => __process_incoming_message(peer_addr, text),
        };
    }
}

fn recv_command(stream: &mut MdswpStream) -> io::Result<Option<c2s::Command>> {
    // Read exactly four bytes which will denote next message length:
    let mut buffer = [0; size_of::<u32>()];
    let read_bytes = stream.read(&mut buffer)?;
    assert!(read_bytes <= size_of::<u32>());
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
    // Prepare a buffer with length of `data_len` bytes:
    let mut buffer = Vec::with_capacity(data_len);
    for _ in 0..data_len { buffer.push(0); }
    // Read exactly `data_len` bytes:
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

fn send_command(stream: &mut MdswpStream, command: s2c::Command) -> io::Result<()> {
    let json = serde_json::to_string(&command);
    // Response encryption:
    let encrypted = encrypt(json.into_bytes());
    if encrypted.len() > u32::MAX as usize {
        return Result::Err(io::Error::new(io::ErrorKind::InvalidInput, "Data too large"));
    }
    stream.write_all(&(encrypted.len() as u32).to_be_bytes())?;
    stream.write_all(&encrypted)?;
    stream.flush()
}

pub(crate) fn handle_err<E>(addr: &SocketAddr, err: E)
    where E: Display
{
    // Log error:
    log(LogLevel::Error, &format!("{}", err));
    // Remove connection:
    let client_info = client_list::remove_connection(&addr)?;
    let mut stream = client_info.stream;
    // Send response back to client:
    let command = s2c::Command::Error { description: err.to_string() };
    let _ = send_command(&mut stream, command);
    let _ = stream.reset();
}

fn __process_fetch(peer_addr: SocketAddr, date_time: DateTime<Utc>) {
    match client_list::get_nickname(&peer_addr).unwrap() {
        Option::None => {
            let stream = client_list::
        },
        Option::Some(_) => {

        }
    }
}

fn __process_login(peer_addr: SocketAddr, is_registering: bool, nickname: String, password: String) {

}

fn __process_send_message(peer_addr: SocketAddr, text: String)