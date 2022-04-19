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

use crate::login::LoginRequest;

use serde::Deserialize;
use serde::Serialize;

/// An enumeration of possible commands that a client can send to a server.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub enum Command {

    /// Command for logging in or registering.
    ///
    /// Server should respond with [`LoginOk`] or [`Error`] message.
    ///
    /// [`Error`]: crate::command::s2c::Command::Error
    /// [`LoginOk`]: crate::command::s2c::Command::LoginOk
    Login (LoginRequest),

    /// Command for sending a message.
    ///
    /// Server should respond with:
    ///
    ///  -  [`Warning`] if given message is not allowed due to regulation rules
    ///  -  [`RecvMessage`] with the same message text if given message is accepted
    ///
    /// [`RecvMessage`]: crate::command::s2c::Command::RecvMessage
    /// [`Warning`]: crate::command::s2c::Command::Warning
    SendMessage (String),
}