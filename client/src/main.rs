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

#[macro_use]
mod util;

use crate::util::{error, io_error, is_err, recv_command};
use crate::util::send_command;

use std::io::BufReader;
use std::io::BufRead;
use std::io::stdin;
use std::io::Stdin;
use std::io::Write;
use std::net::SocketAddr;
use std::process::exit;
use std::sync::RwLock;
use std::thread;

use mdchat_common::command::{c2s, s2c};
use mdchat_common::login::LoginRequest;

use mdswp::MdswpStream;

use once_cell::sync::Lazy;

static mut STDIN_READER: Lazy<BufReader<Stdin>> = Lazy::new(|| BufReader::new(stdin()));
static IS_ERR: Lazy<RwLock<bool>> = Lazy::new(|| RwLock::new(false));

fn main() {
    // IP address and port:
    let mut ip_addr = Option::None;
    let mut port = Option::None;
    // Ask for IP address:
    while matches!(ip_addr, Option::None) {
        let ip = input!("Server IP address: ");
        match ip.trim().parse() {
            Result::Ok(ip) => ip_addr = Option::Some(ip),
            Result::Err(err) => println!("Invalid IP address: {}", err),
        }
    }
    // Ask for port:
    while matches!(port, Option::None) {
        let p = input!("Port: ");
        match p.trim().parse() {
            Result::Ok(p) if p >= 1000 => port = Option::Some(p),
            Result::Ok(p) => println!("Invalid input: Port number {} too small", p),
            Result::Err(err) => println!("Invalid input: {}", err),
        }
    }
    // Ask for username and password:
    let nickname = input!("Nickname: ");
    let password = input!("Password: ");
    // Ask for login or register
    let mut is_registering = Option::None;
    while matches!(is_registering, Option::None) {
        let lor = input!("Login or register? (login is default) [L/R] ");
        match lor.trim() {
            "L" | "l" | "" => is_registering = Option::Some(false),
            "R" | "r" => is_registering = Option::Some(true),
            _other => {}
        }
    }
    // Unwrap IP address and port and build socket address
    let ip_addr = ip_addr.unwrap();
    let port = port.unwrap();
    let is_registering = is_registering.unwrap();
    let socket = SocketAddr::new(ip_addr, port);
    // Connect to server
    let mut conn = match MdswpStream::connect(socket) {
        Result::Ok(stream) => {
            println!("Connected to server successfully. Now you can type your messages");
            stream
        },
        Result::Err(err) => {
            println!("Could not connect: {}", err);
            input!("Press Enter to quit ");
            exit(1);
        }
    };
    // Receiver thread
    let conn_clone = conn.try_clone().unwrap();
    thread::spawn(|| listen_for_incoming(conn_clone));
    // Login command
    let login_request = LoginRequest::new(is_registering, nickname, password);
    let login_command = c2s::Command::Login(login_request);
    // Send login command
    match send_command(&mut conn, login_command) {
        Result::Ok(()) => {}
        Result::Err(err) => io_error(&mut conn, err)
    }

    loop {
        let message = input!("");
        if is_err() { return }
        let command = c2s::Command::SendMessage(message);
        let send_result =  util::send_command(&mut conn, command);
        if let Result::Err(err) = send_result {
            util::io_error(&mut conn, err);
        }
    }
}

fn listen_for_incoming(mut conn: MdswpStream) {
    while !is_err() {
        let command = recv_command(&mut conn);
        let command = match command {
            Result::Ok(cmd) => cmd,
            Result::Err(err) => { io_error(&mut conn, err); return; }
        };
        match command {
            s2c::Command::LoginSuccess => println!("Login successful! Now type your messages."),
            s2c::Command::MessageRecv(message) => println!("{}", message),
            s2c::Command::Warning(description) => println!("WARNING: {}", description),
            s2c::Command::Error(description) => error(&mut conn, description)
        }
    }
}