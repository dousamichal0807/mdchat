use chrono::DateTime;
use chrono::Utc;

use crate::client;
use crate::client_list;
use crate::message_list;
use crate::user_list;

use mdchat_common::message::Message;

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
/// - `date_time`: [`DateTime`], in [`Utc`] time zone, when the message was sent
/// - `text`: text of the message which client sent
///
/// # Returns
///
/// - [`Ok`] if adding into message queue was successful,
/// - [`Err`] containing [`io::Error`] with detailed reason
///
/// [`Message`]: mdchat_util::message::Message
/// [`SocketAddr`]: std::net::SocketAddr
/// [`DateTime`]: chrono::DateTime
/// [`Utc`]: chrono::Utc
/// [`Ok`]: std::result::Result::Ok
/// [`Err`]: std::result::Result::Err
/// [`io::Error`]: std::io::Error
pub fn push(addr: &SocketAddr, date_time: DateTime<Utc>, text: String) -> io::Result<()> {
    // Get nickname. If not present, return None:
    let sender = match client_list::get_nickname(&addr)? {
        Some(nick) => nick,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "The client has not chosen nickname yet",
            ))
        }
    };
    // Push into command queue
    let mut message_queue = MESSAGE_QUEUE.write().unwrap();
    message_queue.push_front(Message {
        sender,
        date_time,
        text,
    });
    drop(message_queue);
    // Everything is OK, return Some:
    Ok(())
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
            Some(message) => handle_msg(message),
            None => thread::sleep(Duration::from_millis(0)),
        }
    }
}

fn pop() -> Option<Message> {
    let mut message_queue = MESSAGE_QUEUE.write().unwrap();
    let message = message_queue.pop_back();
    drop(message_queue);
    return message;
}

fn handle_msg(message: Message) {
    // TODO log(LogKind::Info, &message);
    let msg_id = message_list::push(message.clone());
    client_list::for_each(|(addr, info)| {
        let mut stream = info.connection.try_clone().unwrap();
        match client::send_info(&mut stream, message.to_string()) {
            Ok(()) => user_list::set_last_sent_msg_id(info.nickname.as_ref().unwrap(), msg_id).unwrap(),
            Err(err) => client::handle_err(addr, &err).unwrap(),
        }
    });
}