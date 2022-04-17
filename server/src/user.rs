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