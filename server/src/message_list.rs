use std::collections::BTreeMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

use mdchat_common::message::Message;

static MESSAGE_LIST: Lazy<RwLock<BTreeMap<u64, Message>>> = Lazy::new(|| RwLock::new(BTreeMap::new()));
static LAST_ID: Lazy<RwLock<u64>> = Lazy::new(|| RwLock::new(0));

fn incr_and_get_id() -> u64 {
    let mut last_id = LAST_ID.write().unwrap();
    let val = *last_id;
    let next_id = val + 1;
    *last_id = next_id;
    drop(last_id);
    return next_id;
}

pub fn push(message: Message) -> u64 {
    let message_id = incr_and_get_id();
    let mut message_list = MESSAGE_LIST.write().unwrap();
    message_list.insert(message_id, message);
    return message_id;
}

pub fn for_messages_newer_than<F>(message_id: u64, mut callback: F)
where
    F: FnMut(&u64, &Message)
{
    let message_list = MESSAGE_LIST.read().unwrap();
    let iter = message_list.iter().filter(|(&id, _)| id > message_id);
    for (msg_id, message) in iter {
        callback(msg_id, message);
    }
}