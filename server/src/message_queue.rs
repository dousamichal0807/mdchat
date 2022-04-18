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

use crate::client;
use crate::client_list;
use crate::log;
use crate::message_list;
use crate::user_list;

use mdchat_common::message::Message;

use mdlog::LogLevel;

use once_cell::sync::Lazy;

use std::collections::LinkedList;
use std::io;
use std::net::SocketAddr;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

static MESSAGE_QUEUE: Lazy<RwLock<LinkedList<Message>>> = Lazy::new(|| RwLock::new(LinkedList::new()));

/// Pushes a new [`Message`] into message queue. Note that pushing into queue may not
/// be successful: that's when the passed [`SocketAddr`] is not present in the client
/// list, e.g. client with specified [`SocketAddr`] is not connected.
///
/// # Parameters
///
/// - `addr`: [`SocketAddr`] of the client
/// - `text`: text of the message which client sent
///
/// # Returns
///
/// - [`Result::Ok`] if adding into message queue was successful,
/// - [`Result::Err`] containing [`io::Error`] with detailed reason
pub fn push(addr: &SocketAddr, text: String) -> io::Result<()> {
    let sender = client_list::get_nickname(&addr)?
        .ok_or(io::Error::new(io::ErrorKind::NotFound, "The client have not logged in yet"))?;
    MESSAGE_QUEUE.write().unwrap().push_front(Message::new(sender, Utc::now(), text));
    Result::Ok(())
}

/// Function contaning a loop for continuous message handling. This function should
/// be called in a seperate thread.
///
/// # Usage
///
/// ```
/// use std::thread;
/// use mdchat_server::message_queue;
/// // ...
/// let msg_queue_handler = thread::spawn(message_queue::handle_incoming);
/// ```
pub fn handle_incoming() -> io::Result<()> {
    loop {
        let next = pop();
        match next {
            Option::Some(message) => handle_msg(message),
            Option::None => thread::sleep(Duration::ZERO),
        }
    }
}

#[doc(hidden)]
fn pop() -> Option<Message> {
    MESSAGE_QUEUE.write().unwrap().pop_back()
}

#[doc(hidden)]
fn handle_msg(message: Message) {
    log(LogLevel::Info, &message.to_string());
    let msg_id = message_list::push(message.clone());
    client_list::for_each(|(addr, info)| {
        let mut stream = info.stream.try_clone().unwrap();
        match client::send_info(&mut stream, message.to_string()) {
            Ok(()) => user_list::set_last_sent_msg_id(info.nickname.as_ref().unwrap(), msg_id).unwrap(),
            Err(err) => client::handle_err(addr, &err).unwrap(),
        }
    });
}