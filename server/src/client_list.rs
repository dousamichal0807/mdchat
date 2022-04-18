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

use crate::client::ClientInfo;
use crate::{client, global_config, log, user_list};

use mdswp::MdswpStream;
use once_cell::sync::Lazy;

use std::collections::BTreeMap;
use std::io;
use std::net::SocketAddr;
use std::sync::RwLock;
use std::thread;
use mdlog::LogLevel;

static CLIENT_LIST: Lazy<RwLock<BTreeMap<SocketAddr, ClientInfo>>>
    = Lazy::new(|| RwLock::new(BTreeMap::new()));

#[doc(hidden)]
fn not_connected_error(addr: &SocketAddr) -> io::Error {
    io::Error::new(io::ErrorKind::NotConnected, format!("{} is not connected", addr))
}

/// Returns whether specified client is connected to the server.
pub fn is_connected(addr: &SocketAddr) -> bool {
    CLIENT_LIST.read().unwrap().contains_key(addr)
}

/// Returns whether specified client is logged in.
///
/// # Return value
///
/// Boolean value stating, if specified client is logged in. If specified client is
/// not connected to the server, function will panic.
pub fn is_logged_in(addr: &SocketAddr) -> bool {
    CLIENT_LIST.read().unwrap()
        .get(addr)
        .expect("Specified client is not connected")
        .nickname.is_some()
}

/// Adds a new connection to the client list. If specified client is already
/// registered, this function will panic.
pub fn add_connection(stream: MdswpStream, peer_addr: SocketAddr, thread_handle: thread::JoinHandle<()>) {
    // Create ClientInfo instance
    let client_info = ClientInfo {
        socket_addr: peer_addr,
        nickname: Option::None,
        stream,
        thread_handle
    };
    // Add it to the client list
    let mut client_list = CLIENT_LIST.write().unwrap();
    let previous_value = client_list.insert(peer_addr, client_info);
    assert!(previous_value.is_none(), "Client {} already connected!", socket_addr);

}

/// Removes specified client from the client list and returns a [`ClientInfo`] which
/// was associated with the client. If the client is not connected, method will
/// panic.
pub fn remove_connection(addr: &SocketAddr) -> ClientInfo {
    CLIENT_LIST.write().unwrap()
        .remove(addr)
        .expect("Specified client not connected")
}

/// Returns the nickname of the user which specified client logged into. If client
/// has not logged in yet, method will return [`Option::None`]. If given client is
/// not connected, method will panic.
pub fn get_nickname(addr: &SocketAddr) -> Option<String> {
    CLIENT_LIST.read().unwrap()
        .get(addr).unwrap()
        .nickname.clone()
}

/// Tries to log into the specified account and checks for the validity of the
/// nickname and password.
///
/// # Parameters:
///
///  -  `addr`: client specified by a socket address
///  -  `is_registering`: if client tries to register a new account or wants to log
///     into an existing account
///  -  `nickname`: the account nickname
///  -  `password`: the credential password
///
/// # Panicking
///
/// Panics if given client is not connected.
pub fn login(addr: &SocketAddr, is_registering: bool, nickname: String, password: String) {
    // Check nickname policy:
    if !global_config().is_allowed_nickname(&nickname) {
        client::handle_err(addr, format!("`{}` is not an allowed nickname due to regulations.", nickname))
    }
    // Get if nickname is already registered:
    let is_present = user_list::exists(&nickname);
    // Do something based on if client is trying to register and given nickname
    // already exists
    match (is_registering, is_present) {
        (true, true) => {
            let log_message = format!("Tried to register already existing nickname: `{}`", nickname);
            let client_message = format!("`{}` is already existing user account", nickname);
            client::handle_err(addr, client_message);
            log(LogLevel::Info, &log_message);
        },
        (true, false) => {
            let log_message = format!("Successfully registered and logged in as `{}`", nickname);
            user_list::add_user(nickname.clone(), password);
            *CLIENT_LIST.write().unwrap().get(addr).unwrap().nickname = Option::Some(nickname);
            log(LogLevel::Info, &log_message);
        },
        (false, true) => {
            let result = user_list::verify_password(&nickname, password);
        }
    }
}

/// Calls the same function for all connected clients.
pub fn for_each<F>(f: F)
    where F: FnMut(&SocketAddr, &ClientInfo)
{
    CLIENT_LIST.read().unwrap()
        .iter()
        .for_each(|(sock_addr, client_info)| f(sock_addr, client_info))
}