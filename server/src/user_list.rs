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

pub fn add_user(nickname: String, password: String) -> io::Result<()> {
    if !user::is_valid_nickname(&nickname) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("tried to register with invalid nickname: {:?}", nickname),
        ));
    }
    
    if exists(&nickname) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("User with nickname {} already exists", nickname),
        ));
    }

    let encrypted_password: Vec<u8> = PASSWD_CRYPT.encrypt(password.into_bytes());

    let user_info = UserInfo {
        nickname: nickname.clone(),
        encrypted_password,
        last_sent_msg_id: None,
    };

    let mut user_list = USER_LIST.write().unwrap();
    user_list.insert(nickname, user_info);
    Ok(())
}

pub fn exists(nickname: &str) -> bool {
    let user_list = USER_LIST.read().unwrap();
    let result = user_list.contains_key(nickname);
    drop(user_list);
    return result;
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

pub fn verify_password(nickname: &str, candidate_passwd: String) -> io::Result<bool> {
    let encrypted_candidate: Vec<u8> = PASSWD_CRYPT.encrypt(candidate_passwd.into_bytes());
    let user_list = USER_LIST.read().unwrap();
    let result = match user_list.get(nickname) {
        Some(user_info) => Ok(user_info.encrypted_password == encrypted_candidate),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("User with nickname {} not found", nickname),
        )),
    };
    drop(user_list);
    return result;
}