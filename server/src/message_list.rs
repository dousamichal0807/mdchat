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