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

use serde::Deserialize;
use serde::Serialize;

/// This struct is used by [`s2c::Command::Login`] to provide all needed information
/// needed to log in.
#[derive(Clone, Debug)]
#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    is_registering: bool,
    nickname: String,
    password: String
}

impl LoginRequest {

    /// Creates a new [`LoginRequest`] instance.
    ///
    /// # Parameters
    ///
    ///  -  `is_registering`: should be `true`, if client wants also to register
    ///     a new user rather than just simply logging in
    ///  -  `nickname`: nickname of the user to log into/register
    ///  -  `password`: password of the user to log into/register
    pub fn new(is_registering: bool, nickname: String, password: String) -> Self {
        Self { is_registering, nickname, password }
    }

    /// Creates a new [`LoginRequest`] instance for a client which *does not* want
    /// to create a new user, just to log into an account.
    ///
    /// If such user is not present, server should respond with
    /// [`s2c::Command::Error`].
    ///
    /// [`s2c::Command::Error`]: crate::command::s2c::Command::Error
    pub fn login(nickname: String, password: String) -> Self {
        Self {
            is_registering: false,
            nickname,
            password
        }
    }

    /// Creates a new [`LoginRequest`] instance for a client which wants to register
    /// a new user.
    ///
    /// If such user is present, or a banned nickname is used, server should respond
    /// with [`s2c::Command::Error`].
    ///
    /// [`s2c::Command::Error`]: crate::command::s2c::Command::Error
    pub fn register(nickname: String, password: String) -> Self {
        Self {
            is_registering: true,
            nickname,
            password
        }
    }

    /// Returns whether the client wants to create a new user.
    pub fn is_registering(&self) -> bool {
        self.is_registering
    }

    /// Returns the nickname of the user which the client wants to log
    /// into/register.
    pub fn nickname(&self) -> &String {
        &self.nickname
    }

    /// Returns the password of the user which the client wants to log
    /// into/register.
    pub fn password(&self) -> &String {
        &self.password
    }
}