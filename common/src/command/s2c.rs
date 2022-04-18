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

//! A module for commands that can be sent by server to a client.

use crate::message::Message;

use serde::Deserialize;
use serde::Serialize;

/// An enumeration of possible commands that can be sent by server to a client.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub enum Command {

    /// Represents an error which requires to re-establish a connection.
    ///
    /// If client sends more data, server should reset the connection.
    Error {

        /// Description stating why the error occurred.
        description: String
    },

    /// Represents an important information for the client that needs user's
    /// attention. When [`Warning`] is received connection does not have to be
    /// re-established.
    Warning {

        /// Description stating what happened.
        description: String
    },

    /// Informs client about a new message.
    MessageRecv {

        /// Message which has been sent.
        message: Message
    }
}