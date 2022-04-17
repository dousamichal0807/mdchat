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

use crate::ConfigParseError;
use crate::ConfigParseResult;
use crate::REGEX_WHITESPACE;

use regex::Regex;

use std::collections::HashSet;
use std::num::NonZeroU8;

/// Represents configurability of banning and allowing nicknames of users.
pub struct NicknameFilteringConfig {
    min_len: NonZeroU8,
    max_len: NonZeroU8,
    allowed: HashSet<String>,
    banned: Vec<Regex>
}

impl NicknameFilteringConfig {
    /// Creates a new [`NicknameFilteringConfig`] with default values.
    pub fn new() -> Self {
        Self {
            min_len: unsafe { NonZeroU8::new_unchecked(1) },
            max_len: unsafe { NonZeroU8::new_unchecked(u8::MAX) },
            allowed: HashSet::new(),
            banned: Vec::new(),
        }
    }

    /// Merges `self` with `other` instance in this way:
    ///
    ///  -  minimum and maximum length will be overwritten by `other`'s values
    ///  -  registry of allowed and banned nicknames will be merged with `other`'s
    ///     values
    ///
    /// # Parameters
    ///
    ///  -  `other`: the instance to merge
    pub fn append(&mut self, mut other: NicknameFilteringConfig) {
        self.min_len = other.min_len;
        self.max_len = other.max_len;
        self.allowed = &self.allowed | &other.allowed;
        self.banned.append(&mut other.banned)
    }

    /// Returns minimum nickname length required by the [`NicknameFilteringConfig`]
    /// instance.
    pub fn get_min_len(&self) -> NonZeroU8 {
        self.min_len
    }

    /// Sets minimum nickname length that should be required by the
    /// [`NicknameFilteringConfig`].
    ///
    /// # Parameters
    ///
    ///  -  `min_len`: minimum length to be set
    ///
    /// # Return value
    ///
    ///  -  [`Result::Ok`] if setting the minimum length was successful
    ///  -  [`Result::Err`] if the new minimum length was greater than maximum length
    pub fn set_min_len(&mut self, min_len: NonZeroU8) -> Result<(), String> {
        if min_len > self.max_len {
            Result::Err("Tried to set minimum length greater than maximum length".to_string())
        } else {
            self.min_len = min_len;
            Result::Ok(())
        }
    }

    /// Returns maximum nickname length required by the [`NicknameFilteringConfig`]
    /// instance.
    pub fn get_max_len(&self) -> NonZeroU8 {
        self.max_len
    }

    /// Sets maximum nickname length that should be required by the
    /// [`NicknameFilteringConfig`] instance.
    ///
    /// # Parameters
    ///
    ///  -  `max_len`: maximum length to be set
    ///
    /// # Return value
    ///
    ///  -  [`Result::Ok`] if setting the maximum length was successful
    ///  -  [`Result::Err`] if the new maximum length was lower than minimum length
    pub fn set_max_len(&mut self, max_len: NonZeroU8) -> Result<(), String> {
        if max_len < self.min_len {
            Result::Err("Tried to set maximum length lower than minimum length".to_string())
        } else {
            self.max_len = max_len;
            Result::Ok(())
        }
    }

    /// This methods explicitly allows a specific nickname.
    ///
    /// # Parameters
    ///
    ///  -  `nickname`: nickname that should be always allowed
    pub fn allow(&mut self, nickname: String) {
        self.allowed.insert(nickname);
    }

    /// Method for banning nicknames by a specific [`Regex`] pattern.
    ///
    /// # Parameters
    ///
    ///  -  `regex`: pattern of nicknames which should be banned
    pub fn ban(&mut self, regex: Regex) {
        self.banned.push(regex);
    }

    /// Returns an immutable borrow to the inner [`HashSet`] containing allowed
    /// nicknames.
    pub fn get_allowed_nicknames(&self) -> &HashSet<String> {
        &self.allowed
    }

    /// Returns a mutable borrow to the inner [`HashSet`] containing allowed
    /// nicknames.
    pub fn get_allowed_nicknames_mut(&mut self) -> &mut HashSet<String> {
        &mut self.allowed
    }

    /// Returns an immutable borrow to the inner [`Vec`] of [`Regex`] patterns of
    /// nicknames that should be banned.
    pub fn get_banned_patterns(&self) -> &Vec<Regex> {
        &self.banned
    }

    /// Returns a mutable borrow to the inner [`Vec`] of [`Regex`] patterns of
    /// nicknames that should be banned.
    pub fn get_banned_patterns_mut(&mut self) -> &mut Vec<Regex> {
        &mut self.banned
    }

    /// Returns whether given nickname is allowed to be used.
    pub fn is_allowed(&self, nick: &str) -> bool {
        if self.allowed.contains(nick) { return true }
        for pattern in &self.banned {
            if pattern.is_match(nick) { return false }
        }
        return true
    }

    /// Processes given string as a part of a configuration file.
    ///
    /// # Return value
    ///
    ///  -  [`Result::Ok`] if parsing succeeded
    ///  -  [`Result::Err`] if parsing failed
    pub fn process_string(&mut self, string: &str, rollback_on_error: bool) -> ConfigParseResult<()> {
        if rollback_on_error {
            let mut temp = Self::new();
            temp.process_string(string, false)?;
            self.append(temp);
            return Result::Ok(())
        }
        // Process each line:
        let line_num = 1u32;
        for line in string.lines() {
            self.process_line(line)
                .map_err(|err| ConfigParseError::syntax_error(String::new(), line_num, err))?;
        }
        Result::Ok(())
    }

    /// Processes a single line of configuration file. If a newline character is
    /// found after the trim of the line, method will panic.
    ///
    /// # Panicking
    ///
    /// Panics if a newline character is in the middle of given string.
    ///
    /// # Return value
    ///
    ///  -  [`Result::Ok`] if parsing succeeded
    ///  -  [`Result::Err`] if parsing failed
    pub fn process_line(&mut self, line: &str) -> Result<(), String> {
        // Trim line and ensure that there is no newline character:
        let line = line.trim();
        assert!(!line.contains("\n"), "Passed multi-line string to process_line");
        // Do not process empty lines any further:
        if line.is_empty() { return Result::Ok(()) }
        // Split command and argument:
        let split: Vec<&str> = REGEX_WHITESPACE.splitn(line, 2).collect();
        let command = split[0];
        let arg = split.get(1).map(|x| x.to_owned());
        // Parse based on the command
        match command {
            "allow" => self.__process_allow(arg),
            "ban" => self.__process_ban(arg),
            "max-length" => self.__process_max_length(arg),
            "min-length" => self.__process_min_length(arg),
            other => Result::Err(format!("`nickname {}`: unknown sub-command", other))
        }
    }

    #[doc(hidden)]
    fn __process_allow(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("An argument was expected after `nickname allow`".to_string())
            .map(|str| { self.allow(str.to_string()); })
    }

    #[doc(hidden)]
    fn __process_ban(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("An argument was expected after `nickname ban-pattern`".to_string())
            .and_then(|arg| Regex::new(arg)
                .map_err(|err| format!("Could not parse given regex after `nickname ban-pattern`: {}", err)))
            .map(|regex| self.ban(regex))
    }

    #[doc(hidden)]
    fn __process_min_length(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("An argument was expected after `nickname min-length`".to_string())
            .and_then(|arg| arg.parse()
                .map_err(|err| format!("A number between 1 and 255 was expected after `nickname min-length`: {}", err)))
            .and_then(|arg| self.set_min_len(arg))
    }

    #[doc(hidden)]
    fn __process_max_length(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("An argument was expected after `nickname max-length`".to_string())
            .and_then(|arg| arg.parse()
                .map_err(|err| format!("A number between 1 and 255 was expected after `nickname max-length`: {}", err)))
            .and_then(|arg| self.set_max_len(arg))
    }
}