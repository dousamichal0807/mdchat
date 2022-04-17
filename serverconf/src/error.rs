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
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

use thiserror::Error;

/// A type alias for result of parsing a configuration file.
pub type ConfigParseResult<T> = Result<T, ConfigParseError>;

/// Represents an config caused by inability to parse a configuration file.
#[derive(Error, Debug)]
#[error("{file_path}: {kind}")]
pub struct ConfigParseError {
    file_path: String,
    kind: ConfigParseErrorKind,
}

impl ConfigParseError {
    /// Creates a new [`ConfigParseError`] instance from given file path and error kind.
    pub fn new(file_path: String, kind: ConfigParseErrorKind) -> Self {
        Self { file_path, kind }
    }

    /// Creates a new [`ConfigParseError`] instance with [`Syntax`] error kind using
    /// given given file path, line number and error description.
    ///
    /// [`Syntax`]: ConfigParseErrorKind::Syntax
    pub fn syntax_error(file_path: String, line_num: u32, description: String) -> Self {
        Self {
            file_path,
            kind: ConfigParseErrorKind::Syntax { line_num, description }
        }
    }

    /// Creates a new [`ConfigParseError`] instance with [`Io`] error kind using
    /// given file path and [`std::io::Error`].
    ///
    /// [`Io`]: ConfigParseErrorKind::Io
    pub fn io_error(file_path: String, io_error: io::Error) -> Self {
        Self {
            file_path,
            kind: ConfigParseErrorKind::Io { io_error }
        }
    }

    /// Returns the path of the file which could not be parsed.
    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Returns the reason why parsing configuration file did not succeed.
    pub fn kind(&self) -> &ConfigParseErrorKind {
        &self.kind
    }
}

/// Represents a reason why a configuration file could not be parsed.
///
/// See this enum's variants for more information.
#[derive(Error, Debug)]
pub enum ConfigParseErrorKind {

    /// Represents an error caused by an unsuccessful I/O operation.
    Io {
        /// The underlying I/O error
        #[from]
        io_error: io::Error
    },

    /// Represents an error caused by invalid syntax of the configuration file which
    /// has been read.
    Syntax {
        /// Line number, where error was found
        line_num: u32,
        /// Description of the error
        description: String
    }
}

impl Display for ConfigParseErrorKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Io { io_error } => write!(f, "I/O error: {}", io_error),
            Self::Syntax { line_num, description } => write!(f, "Line {}: {}", line_num, description)
        }
    }
}
