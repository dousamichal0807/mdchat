/*
 * Copyright (c) 2022 Michal Douša. All rights reserved.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::fmt;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;

#[derive(Clone, Debug)]
pub struct Message {
    pub sender: String,
    pub date_time: DateTime<Utc>,
    pub text: String,
}

impl fmt::Display for Message {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{} @ {}: {}",
               self.sender,
               self.date_time.with_timezone(&Local).format("%F %T"),
               self.text
        )
    }
}