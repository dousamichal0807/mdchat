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
use chrono::Local;
use chrono::Utc;

use serde::Deserialize;
use serde::Serialize;

use std::fmt;

/// A structure representing a message in a chat.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Message {
    sender: String,
    date_time: DateTime<Utc>,
    text: String,
}

impl Message {
    /// Creates a new [`Message`] instance
    ///
    /// # Parameters
    ///
    ///  -  `sender`: nickname of user, which has sent the message
    ///  -  `date_time`: date and time, when the message was sent
    ///  -  `text`: content of the message
    pub fn new(sender: String, date_time: DateTime<Utc>, text: String) {
        Self { sender, date_time, text }
    }

    /// Returns the nickname of the user who sent the message.
    pub fn sender(&self) -> &String {
        &self.sender
    }

    /// Returns date and time when the message was sent.
    pub fn date_time(&self) -> &DateTime<Utc> {
        &self.date_time
    }

    /// Returns the content of the message.
    pub fn text(&self) -> &String {
        &self.text
    }
}

impl fmt::Display for Message {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}@{}: {}",
               self.sender,
               self.date_time.with_timezone(&Local).to_rfc3339(),
               self.text
        )
    }
}