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

use std::collections::BTreeMap;
use std::io;
use std::net::SocketAddr;
use std::sync::RwLock;
use std::thread;

use mdswp::MdswpStream;
use once_cell::sync::Lazy;

use crate::client::ClientInfo;
use crate::user_list;

static CLIENT_LIST: Lazy<RwLock<BTreeMap<SocketAddr, ClientInfo>>>
    = Lazy::new(|| RwLock::new(BTreeMap::new()));

fn not_connected_error(addr: &SocketAddr) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotConnected,
        format!("{} is not connected", addr)
    )
}

pub fn is_connected(addr: &SocketAddr) -> bool {
    let client_list = CLIENT_LIST.read().unwrap();
    let is_connected = client_list.contains_key(addr);
    drop(client_list);
    return is_connected;
}

pub fn add_connection(
    connection: MdswpStream,
    thread_handle: thread::JoinHandle<io::Result<()>>
) {
    let addr = connection.peer_addr().unwrap();
    if is_connected(&addr) { unreachable!() }

    let client_info = ClientInfo { socket_addr: addr, nickname: None, connection, thread_handle };
    let mut client_list = CLIENT_LIST.write().unwrap();
    client_list.insert(addr, client_info);
    drop(client_list);
}

pub fn remove_connection(addr: &SocketAddr) -> io::Result<ClientInfo> {
    let mut client_list = CLIENT_LIST.write().unwrap();
    let client_info = client_list.remove(addr);
    drop(client_list);
    match client_info {
        Some(client_info) => Ok(client_info),
        None => Err(not_connected_error(addr))
    }
}

pub fn get_nickname(addr: &SocketAddr) -> io::Result<Option<String>> {
    let client_list = CLIENT_LIST.read().unwrap();
    let client_info = client_list.get(addr);
    let nickname = match client_info {
        Some(i) => i.nickname.clone(),
        None => return Err(not_connected_error(addr))
    };
    drop(client_list);
    Ok(nickname)
}

pub fn login(addr: &SocketAddr, nickname: String, password: String) -> io::Result<()> {
    match user_list::verify_password(&nickname, password.clone()) {
        Ok(false) => return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("tried invalid password for user {}", nickname)
        )),
        Ok(true) => {}, // Do nothing, continue
        Err(_) => user_list::add_user(nickname.clone(), password.clone()).unwrap()
        // user is not in the list yet
    };

    let mut client_list = CLIENT_LIST.write().unwrap();
    let result = match client_list.get_mut(addr) {
        Some(client_info) => {client_info.nickname = Some(nickname); Ok(()) },
        None => Err(io::Error::new(
            io::ErrorKind::NotConnected,
            format!("{} is not connected now", addr)
        ))
    };

    drop(client_list);
    return result;
}

pub fn for_each<F>(f: F) where F: FnMut((&SocketAddr, &ClientInfo)) {
    let client_list = CLIENT_LIST.read().unwrap();
    client_list.iter().for_each(f);
    drop(client_list);
}