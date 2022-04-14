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
    /// Creates a new [`ConfigParseError`] instance from given file path and config kind.
    pub fn new(file_path: String, kind: ConfigParseErrorKind) -> Self {
        Self { file_path, kind }
    }

    /// Creates a new [`ConfigParseError`] instance with [`Syntax`] config kind using
    /// given given file path, line number and config description.
    ///
    /// [`Syntax`]: ConfigParseErrorKind::Syntax
    pub fn syntax_error(file_path: String, line_num: u32, description: String) -> Self {
        Self {
            file_path,
            kind: ConfigParseErrorKind::Syntax { line_num, description }
        }
    }

    /// Creates a new [`ConfigParseError`] instance with [`Io`] config kind using
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

    /// Represents an config caused by an unsuccessful I/O operation.
    #[error("I/O error: {io_error}")]
    Io {
        /// The underlying I/O config
        #[from]
        io_error: io::Error
    },

    /// Represents an config caused by invalid syntax of the configuration file which
    /// has been read.
    #[error("Line {line_num}: {description}")]
    Syntax {
        /// Line number, where config was found
        line_num: u32,
        /// Description of the config
        description: String
    }
}
