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
use crate::log;
use crate::message_list;
use crate::user_list;

use mdchat_common::command::s2c;
use mdchat_common::message::Message;

use mdlog::LogLevel;

use once_cell::sync::Lazy;

use std::collections::LinkedList;
use std::io;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

static MESSAGE_QUEUE: Lazy<RwLock<LinkedList<Message>>> = Lazy::new(|| RwLock::new(LinkedList::new()));

/// Pushes a new [`Message`] into message queue.
///
/// # Parameters
///
/// - `sender`: nickname of the user who sent the message
/// - `text`: text of the message which client sent
///
/// # Returns
pub fn push(sender: String, text: String) {
    let message = Message::new(sender, Utc::now(), text);
    MESSAGE_QUEUE.write().unwrap().push_front(message);
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
    // Log that message is being processed:
    let log_message = format!("A new message is being processed: {:?}", message);
    log(LogLevel::Debug, &log_message);
    // Add message to message list
    let msg_id = message_list::push(message.clone());
    // Send message to all clients that are logged in:
    let command = s2c::Command::MessageRecv(message);
    client_list::for_each(|_, client| match client.nickname() {
        Option::None => {},
        Option::Some(nickname) => match client.send_command(command.clone()) {
            Result::Ok(()) => user_list::set_last_sent_msg_id(&nickname, msg_id),
            Result::Err(err) => client.error(err.to_string()),
        }
    });
}