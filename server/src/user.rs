#[derive(Clone)]
pub struct UserInfo {
    pub nickname: String,
    pub encrypted_password: Vec<u8>,
    pub last_sent_msg_id: Option<u64>,
}

pub fn is_valid_nickname(candidate: &str) -> bool {
    if candidate.len() == 0 { return false }
    else { candidate.as_bytes().iter().all(|byte| (0x20u8..=0x7Eu8).contains(byte)) }
}