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
use std::sync::RwLock;
use once_cell::sync::Lazy;

use mdcrypt::algorithms::Sha512;
use mdcrypt::Encrypt;

use crate::user;
use crate::user::UserInfo;

static USER_LIST: Lazy<RwLock<BTreeMap<String, UserInfo>>> = Lazy::new(|| RwLock::new(BTreeMap::new()));
static PASSWD_CRYPT: Lazy<Sha512> = Lazy::new(|| Sha512::default());

/// Adds a new user into the list of users.
pub fn add_user(nickname: String, password: String) {
    // Encrypt password
    let encrypted_password: Vec<u8> = PASSWD_CRYPT.encrypt(password.into_bytes());
    // Create UserInfo instance
    let user_info = UserInfo {
        nickname: nickname.clone(),
        encrypted_password,
        last_sent_msg_id: None,
    };
    // Put it into user list:
    let previous_value = USER_LIST.write().unwrap().insert(nickname, user_info);
    assert!(previous_value.is_none(), "Specified user already exists");
}

/// Returns if given user already exists.
pub fn exists(nickname: &str) -> bool {
    USER_LIST.read().unwrap().contains_key(nickname)
}

pub fn get_last_sent_msg_id(nickname: &str) -> io::Result<Option<u64>> {
    let mut user_list = USER_LIST.write().unwrap();
    let result = match user_list.get_mut(nickname) {
        Some(user_info) => Ok(user_info.last_sent_msg_id),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("User with nickname {} not found", nickname),
        )),
    };
    drop(user_list);
    return result;
}

pub fn set_last_sent_msg_id(nickname: &str, last_sent_msg_id: u64) -> io::Result<()> {
    let mut user_list = USER_LIST.write().unwrap();
    let result = match user_list.get_mut(nickname) {
        Some(user_info) => {
            user_info.last_sent_msg_id = Some(last_sent_msg_id);
            Ok(())
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("User with nickname {} not found", nickname),
        )),
    };
    drop(user_list);
    return result;
}

pub fn verify_password(nickname: &str, candidate_passwd: String) -> bool {
    let encrypted_candidate: Vec<u8> = PASSWD_CRYPT.encrypt(candidate_passwd.into_bytes());
    encrypted_candidate == USER_LIST.read().unwrap().get(nickname).unwrap().encrypted_password
}