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

//! `mdchat_serverconf` is a dependency of `mdchat_server`. This dependency allows
//! the server to be configurable. This dependency is automatically included when
//! compiling the server. See `mdchat_serverconf`'s README for more information
//! about possible configurability.

#[doc(hidden)]
mod error;

pub mod ip;
pub mod message;
pub mod nickname;

pub use crate::error::ConfigParseError;
pub use crate::error::ConfigParseResult;
pub use crate::error::ConfigParseErrorKind;
pub use crate::ip::IpFilteringConfig;
pub use crate::message::MessageFilteringConfig;
pub use crate::nickname::NicknameFilteringConfig;

use mdlog::loggers::TextLogger;

use once_cell::sync::Lazy;

use regex::Regex;

use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Stdout;
use std::io::stdout;
use std::net::AddrParseError;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::path::Path;
use std::sync::RwLock;
use mdlog::LogLevel;

static REGEX_WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

/// Represents a complete configuration of the server.
pub struct Config {
    ip_filtering: RwLock<IpFilteringConfig>,
    message_filtering: RwLock<MessageFilteringConfig>,
    nickname_filtering: RwLock<NicknameFilteringConfig>,
    listen_sock_addrs: RwLock<HashSet<SocketAddr>>,
    logger: RwLock<TextLogger<Stdout>>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Creates a new [`Config`] instance with default values.
    ///
    /// Calling this constructor is same as using [`Default`] trait implementation.
    pub fn new() -> Self {
        Self {
            ip_filtering: RwLock::new(IpFilteringConfig::new()),
            listen_sock_addrs: RwLock::new(HashSet::new()),
            logger: RwLock::new(TextLogger::new(LogLevel::Debug, stdout())),
            message_filtering: RwLock::new(MessageFilteringConfig::new()),
            nickname_filtering: RwLock::new(NicknameFilteringConfig::new())
        }
    }

    /// Appends other [`Config`] instance to `self`.
    ///
    /// Fields, which are not collections, will get overwritten by the `other`
    /// instance. Fields which are collections, will be merged with `self`'s fields.
    pub fn append(&self, other: Self) {
        // IP address, nickname and message filtering
        self.ip_filtering.write().unwrap().append(&*other.ip_filtering.read().unwrap());
        self.message_filtering.write().unwrap().append(other.message_filtering.into_inner().unwrap());
        self.nickname_filtering.write().unwrap().append(other.nickname_filtering.into_inner().unwrap());
        // Listener socket addresses
        let mut self_listen = self.listen_sock_addrs.write().unwrap();
        let other_listen = other.listen_sock_addrs.read().unwrap();
        *self_listen = &*self_listen | &*other_listen;
        // Nickname filtering
    }

    /// Returns a read-write lock to the [`IpFilteringConfig`] instance of the
    /// [`Config`].
    pub fn ip_filtering(&self) -> &RwLock<IpFilteringConfig> {
        &self.ip_filtering
    }

    /// Returns a read-write lock to the [`MessageFilteringConfig`] instance of the
    /// [`Config`].
    pub fn message_filtering(&self) -> &RwLock<MessageFilteringConfig> {
        &self.message_filtering
    }
    /// Returns a read-write lock to the [`NicknameFilteringConfig`] instance of the
    /// [`Config`].
    pub fn nickname_filtering(&self) -> &RwLock<NicknameFilteringConfig> {
        &self.nickname_filtering
    }

    /// Returns a read-write lock to the [`TextLogger`] printing to [`stdout`]
    pub fn logger(&self) -> &RwLock<TextLogger<Stdout>> {
        &self.logger
    }

    pub fn process_file<P>(&self, file_path: P, rollback_on_error: bool) -> ConfigParseResult<()>
        where P: AsRef<Path> + Display,
    {
        if rollback_on_error {
            // Provide a rollback when config occurs by saving into temporary
            // `Config` instance without rollbacking:
            let temp_config = Config::default();
            temp_config.process_file(file_path, false)?;
            // If loading is successful, merge configs and return Ok:
            self.append(temp_config);
            return Result::Ok(())
        }
        // Convert file path to string
        let file_name = file_path.to_string();
        // Convertor from io::Error into ConfigParseError
        let convert_io_err = |io_error| ConfigParseError::io_error(file_name.clone(), io_error);
        // Open the file for reading
        let file = File::open(&file_path).map_err(convert_io_err)?;
        // Create buffered reader
        let reader = BufReader::new(file);
        // Store line number:
        let mut line_num = 1u32;
        // Read file line by line:
        for line in reader.lines() {
            line.map_err(convert_io_err)
                .and_then(|line| self.process_line(&line)
                    .map_err(|desc| ConfigParseError::syntax_error(file_name.clone(), line_num, desc))
                )?;
            line_num += 1;
        }
        Result::Ok(())
    }

    /// Processes given string as a part of a configuration file.
    ///
    /// # Return value
    ///
    ///  -  [`Result::Ok`] if parsing succeeded
    ///  -  [`Result::Err`] if parsing failed
    pub fn process_string(&self, string: &str) -> ConfigParseResult<()> {
        // Process line by line:
        let mut line_num = 1u32;
        for line in string.lines() {
            // If there is an config return immediately (`?` operator):
            self.process_line(line)
                .map_err(|desc| ConfigParseError::syntax_error(String::new(), line_num, desc))?;
            line_num += 1;
        }
        // If processing was successful, return Ok:
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
    pub fn process_line(&self, line: &str) -> Result<(), String> {
        // Trim whitespaces
        let line = line.trim();
        // If there is newline character, panic!
        assert!(!line.contains("\n"),
                "Config::process_line(): given string contains newline character");
        // If the line is empty or there is a comment, return immediately:
        if line.is_empty() || line.starts_with("#") {
            return Result::Ok(())
        }
        // If it is not an empty line or a comment, parse it:
        // Split once by space:
        let split: Vec<&str> = REGEX_WHITESPACE.splitn(line.trim(), 2).collect();
        // Separate and trim option and argument. Trim argument and if the argument
        // is empty, discard it.
        let option = split[0].trim();
        let arg = split.get(1).map(|s| s.trim());
        // Based on the option parse it differently:
        match option {
            "ip" => self.__process_ip_command(arg),
            "message" => self.__process_message_command(arg),
            "listen" => self.__process_listen_command(arg),
            "nickname" => self.__process_nickname_command(arg),
            other => Result::Err(format!("`{}` is an invalid option", other))
        }
    }

    #[doc(hidden)]
    fn __process_ip_command(&self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Sub-command was expected after `ip`".to_string())
            .and_then(|arg| self.ip_filtering.write().unwrap().process_line(arg))
    }

    #[doc(hidden)]
    fn __process_message_command(&self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Sub-command was expected after `message`".to_string())
            .and_then(|arg| self.message_filtering.write().unwrap().process_line(arg))
    }

    #[doc(hidden)]
    fn __process_nickname_command(&self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Sub-command was expected after `nickname`".to_string())
            .and_then(|arg| self.nickname_filtering.write().unwrap().process_line(arg))
    }

    #[doc(hidden)]
    fn __process_listen_command(&self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Socket address was expected after `listen`".to_string())
            .and_then(|arg| arg.parse().map_err(|err|
                format!("Invalid socket address after `listen`: {}", err)))
            .map(|sockaddr| { self.listen_sock_addrs.write().unwrap().insert(sockaddr); })

    }

    pub fn is_allowed_ip_addr(&self, addr: &IpAddr) -> bool {
        self.ip_filtering.read().unwrap().is_allowed(addr)
    }

    pub fn is_allowed_message_text(&self, text: &str) -> bool {
        self.message_filtering.read().unwrap().is_allowed(text)
    }

    pub fn is_allowed_nickname(&self, nickname: &str) -> bool {
        self.nickname_filtering.read().unwrap().is_allowed(nickname)
    }

    pub fn listen_sock_addrs(&self) -> &RwLock<HashSet<SocketAddr>> {
        &self.listen_sock_addrs
    }
}