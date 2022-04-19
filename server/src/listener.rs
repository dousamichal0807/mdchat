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

use crate::client::Client;
use crate::client_list;
use crate::log;

use mdlog::LogLevel;

use mdswp::MdswpListener;
use mdswp::MdswpStream;

use std::net::SocketAddr;
use std::thread;

/// Method for infinite accepting a connection. This is a blocking method to be run
/// in a separate thread.
///
/// # Parameters:
///
///  -  `listener`: the listener to listen on
pub fn listen(listener: MdswpListener) {
    for client in listener.incoming() {
        match client {
            Result::Ok((stream, peer_addr)) => __handle_conn(stream, peer_addr),
            Result::Err(err) => log(LogLevel::Warning,
                &format!("A client could not connect to the server: {}", err))
        }
    }
}

#[doc(hidden)]
fn __handle_conn(stream: MdswpStream, peer_addr: SocketAddr) {
    let client = Client::new(stream);
    // Run a thread for the client
    thread::Builder::new()
        .name(format!("Client thread for {}", peer_addr))
        .spawn(cls_clone!(client -> move || client.client_thread()))
        .unwrap();
    // Add new client stream to the clients:
    client_list::add_connection(client);
}