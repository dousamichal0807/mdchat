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

use once_cell::sync::Lazy;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::RwLock;

#[doc(hidden)]
static CLIENT_LIST: Lazy<RwLock<BTreeMap<SocketAddr, Arc<Client>>>>
    = Lazy::new(|| RwLock::new(BTreeMap::new()));

/// Adds a new connection to the client list. If specified client is already
/// registered, this function will panic.
pub fn add_connection(client: Arc<Client>) {
    let addr = *client.socket_addr();
    // Add it to the client list
    let mut client_list = CLIENT_LIST.write().unwrap();
    let previous_value = client_list.insert(addr, client);
    assert!(previous_value.is_none(), "Client {} already connected!", addr);

}

/// Removes specified client from the client list and returns a [`ClientInfo`] which
/// was associated with the client. If the client is not connected, method will
/// panic.
pub fn remove_connection(addr: &SocketAddr) -> Arc<Client> {
    CLIENT_LIST.write().unwrap()
        .remove(addr)
        .expect("Specified client not connected")
}

/// Calls the same function for all connected clients.
pub fn for_each<F>(mut f: F)
    where F: FnMut(&SocketAddr, &Client)
{
    CLIENT_LIST.read().unwrap()
        .iter()
        .for_each(|(sock_addr, client_info)| f(sock_addr, client_info))
}