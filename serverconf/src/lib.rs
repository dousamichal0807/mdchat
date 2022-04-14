mod error;
mod ip;

pub use self::error::ConfigParseError;
pub use self::error::ConfigParseResult;
pub use self::error::ConfigParseErrorKind;

use mdlog::CompositeLogger;

use once_cell::sync::Lazy;

use regex::Regex;

use std::cmp::max;
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::net::AddrParseError;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::num::NonZeroU16;
use std::num::NonZeroU8;
use std::ops::RangeInclusive;
use std::path::Path;
use crate::ip::IpConfig;

macro_rules! assert_valid_config {
    ($this:expr) => {
        assert!($this.is_valid_config(), "Tried to query config in an invalid state");
    }
}

static REGEX_WHITESPACE: Lazy<Regex> = Lazy::new(|| Regex::new("[ \t]+").unwrap());

pub struct Config {
    ip_config: IpConfig,
    listen_sock_addrs: HashSet<SocketAddr>,
    logger: CompositeLogger,
    message_max_len: NonZeroU16,
    message_min_len: NonZeroU16,
    nicknames_banned: Vec<Regex>,
    nicknames_allowed: HashSet<String>,
    nickname_max_len: NonZeroU8,
    nickname_min_len: NonZeroU8,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new [`Config`] instance with default values.
    pub(crate) fn new() -> Self {
        Self {
            ip_config: IpConfig::new(),
            listen_sock_addrs: HashSet::new(),
            logger: CompositeLogger::new(),
            message_max_len: unsafe { NonZeroU16::new_unchecked(u16::MAX) },
            message_min_len: unsafe { NonZeroU16::new_unchecked(1) },
            nicknames_banned: Vec::new(),
            nicknames_allowed: HashSet::new(),
            nickname_max_len: unsafe { NonZeroU8::new_unchecked(u8::MAX) },
            nickname_min_len: unsafe { NonZeroU8::new_unchecked(1) },
        }
    }

    /// Appends other [`Config`] instance to `self`. Fields, which are not
    /// collections, will get overwritten by the other instance. Fields
    /// which collections are, will be merged with `self`'s fields.
    pub fn append(&mut self, mut other: Self) {
        self.ip_config.append(&other.ip_config);
        self.listen_sock_addrs = &self.listen_sock_addrs | &other.listen_sock_addrs;
        //self.logger.append(&mut other.logger);
        self.message_max_len = other.message_max_len;
        self.message_min_len = other.message_min_len;
        self.nicknames_banned.append(&mut other.nicknames_banned);
        self.nicknames_allowed = &self.nicknames_allowed | &other.nicknames_allowed;
        self.nickname_max_len = other.nickname_max_len;
        self.nickname_min_len = other.nickname_min_len;
    }

    pub fn process_file<P>(&mut self, file_path: P, rollback_on_error: bool) -> ConfigParseResult<()>
        where P: AsRef<Path> + Display,
    {
        if rollback_on_error {
            // Provide a rollback when config occurs by saving into temporary
            // `Config` instance without rollbacking:
            let mut temp_config = Config::default();
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

    /// Parses a string of characters.
    pub fn process_string(&mut self, string: &str) -> ConfigParseResult<()> {
        // Process line by line:
        let mut line_num = 1u32;
        for line in string.lines() {
            // If there is an config return immediately (`?` operator):
            self.process_line(line)
                .map_err(|desc| ConfigParseError::syntax_error("IN".to_string(), line_num, desc))?;
            line_num += 1;
        }
        // If processing was successful, return Ok:
        Result::Ok(())
    }

    pub fn process_line(&mut self, line: &str) -> Result<(), String> {
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
            "ip-allow" => self.__process_ip_allow(arg),
            "ip-ban" => self.__process_ip_ban(arg),
            "ip-ban-range" => self.__process_ip_ban_range(arg),
            "nickname-allow" => self.__process_nickname_allow(arg),
            "nickname-ban" => self.__process_nickname_ban(arg),
            "listen" => self.__process_listen(arg),
            "message-length-max" => self.__process_msg_len_max(arg),
            other => Result::Err(format!("`{}` is an invalid option", other))
        }
    }

    #[doc(hidden)]
    fn __process_ip_allow(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("IP address was expected after `ip-allow`".to_string())
            .and_then(|arg| arg.parse().map_err(|err|
                format!("Invalid IP address after `ip-allow`: {}", err)))
            .map(|ip_addr| { self.ip_config.allow(&ip_addr); })
    }

    #[doc(hidden)]
    fn __process_ip_ban(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("IP address was expected after `ip-ban`".to_string())
            .and_then(|arg| arg.parse().map_err(|err|
                format!("Invalid IP address after `ip-ban`: {}", err)))
            .map(|ip_addr| { self.ip_config.ban(&ip_addr); })
    }

    #[doc(hidden)]
    fn __process_ip_ban_range(&mut self, arg: Option<&str>) -> Result<(), String> {
        // Constants
        const ERR_GENERIC: &str = "Two IP addresses separated by space were expected after `ip-ban-range`";
        // Functions
        fn err_invalid_ip(ip: &str, err: AddrParseError) -> String {
            format!("`ip-ban-range`: `{}` is not a valid IP address: {}", ip, err)
        }
        // Argument must be present
        let arg = arg.ok_or(ERR_GENERIC.to_string())?;
        // Split by space and check we have two parts:
        let split: Vec<&str> = REGEX_WHITESPACE.split(arg.trim()).collect();
        if split.len() != 2 { return Result::Err(ERR_GENERIC.to_string()) }
        // Parse both parts:
        let range_from = split[0].trim();
        let range_to = split[1].trim();
        let range_from_parsed = range_from.parse().map_err(|err| err_invalid_ip(range_from, err))?;
        let range_to_parsed = range_to.parse().map_err(|err| err_invalid_ip(range_to, err))?;
        // Ban:
        self.ip_config.ban_range(&range_from_parsed, &range_to_parsed)
            .map_err(|err| format!("`ip-ban-range`: {}", err))
    }

    #[doc(hidden)]
    fn __process_nickname_allow(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Regular expression was expected after `nickname-ban`".to_string())
            .map(|nickname| { self.nicknames_allowed.insert(nickname.to_string()); })
    }

    #[doc(hidden)]
    fn __process_nickname_ban(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Regular expression was expected after `nickname-ban`".to_string())
            .and_then(|arg| arg.parse().map_err(|err|
                format!("Invalid regular expression after `nickname-ban`: {}", err)))
            .map(|regex| self.nicknames_banned.push(regex))
    }

    #[doc(hidden)]
    fn __process_listen(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("Socket address was expected after `listen`".to_string())
            .and_then(|arg| arg.parse().map_err(|err|
                format!("Invalid socket address after `listen`: {}", err)))
            .map(|sockaddr| { self.listen_sock_addrs.insert(sockaddr); })

    }

    #[doc(hidden)]
    fn __process_msg_len_max(&mut self, arg: Option<&str>) -> Result<(), String> {
        arg.ok_or("`nolimit` or a number was expected after `message_length_max`".to_string())
            .and_then(|arg| match arg {
                "nolimit" => {
                    self.message_max_len = unsafe { NonZeroU16::new_unchecked(u16::MAX) };
                    Result::Ok(())
                },
                _ => arg.parse()
                    .map_err(|_| "`nolimit` or a number was expected after `message_length_max`".to_string())
                    .map(|num| { self.message_max_len = num; })
            })
    }

    pub fn is_valid_config(&self) -> bool {
        self.nickname_min_len <= self.nickname_max_len &&
            self.message_min_len <= self.message_max_len
    }

    pub fn is_allowed_nickname(&self, nickname: &str) -> bool {
        assert_valid_config!(self);
        // Is in allowed nicknames?
        if self.nicknames_allowed.contains(nickname) { return true }
        // Is nickname length in bounds?
        if nickname.len() < self.nickname_min_len.get().into() { return false }
        if nickname.len() > self.nickname_max_len.get().into() { return false }
        // Does nickname match any banned patterns?
        for pattern in &self.nicknames_banned {
            if pattern.is_match(nickname) { return false }
        }
        // If all this succeeded, return true
        return true
    }

    pub fn is_allowed_ip_addr(&self, addr: &IpAddr) -> bool {
        assert_valid_config!(self);
        self.ip_config.is_allowed_addr(addr)
    }

    pub fn listen_sock_addrs(&self) -> &HashSet<SocketAddr> {
        assert_valid_config!(self);
        &self.listen_sock_addrs
    }
}