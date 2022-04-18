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

//! A module for commands that can be sent to server by client.

use chrono::DateTime;
use chrono::Utc;

/// An enumeration of possible commands that a client can send to a server.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub enum Command {

    /// Command for fetching all messages which were sent since given date and time.
    ///
    /// Server should respond with [`RecvMessage`] command(s) if a new message(s)
    /// were sent after given date and time.
    ///
    /// [`RecvMessage`]: crate::command::s2c::Command::RecvMessage
    Fetch (
        /// The date and time. Server will send all messages which were sent later.
        DateTime<Utc>
    ),

    /// Command for logging in or registering.
    ///
    /// Server should respond with [`LoginOk`] or [`Error`] message.
    ///
    /// [`Error`]: crate::command::s2c::Command::Error
    /// [`LoginOk`]: crate::command::s2c::Command::LoginOk
    Login {
        /// Boolean indicating if given request is to register a new user (`true`)
        /// or just login (`false`).
        is_registering: bool,
        /// Nickname of the user which client logs into/registers.
        nickname: String,
        /// Password of the user which client logs into/registers.
        password: String
    },

    /// Command for sending a message.
    SendMessage (
        /// The text to be sent as a message.
        String
    ),
}