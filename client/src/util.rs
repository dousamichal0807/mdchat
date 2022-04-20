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

use mdchat_common::command::c2s;
use mdchat_common::command::s2c;

use mdswp::MdswpStream;

use std::error::Error;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process::exit;

/// Flushes `stdout`.
macro_rules! flush {
    () => {
        std::io::stdout().flush().unwrap();
    }
}

/// Prints specified message to `stdout` and asks user for `stdin` input.
///
/// # Example
///
/// ```rust
/// let name = input!("What's your name? ");
/// let color =  input!("So your name is {} and your favorite color is? ", name);
/// println!("{}, your favorite color is {}!", name, color);
/// ```
///
/// Example output:
///
/// ```plain
/// What's your name? Michal
/// So your name is Michal and your favorite color is? blue
/// Michal, your favorite color is blue!
/// ```
macro_rules! input {
    ($str:literal $(, $arg:expr)*) => { unsafe {
        let mut buf = String::new();
        print!($str $(, $arg)*);
        flush!();
        crate::STDIN_READER.read_line(&mut buf).unwrap();
        buf.trim().to_string()
    } }
}

/// Encrypts and sends a [`c2s::Command`] using given [`MdswpStream`].
pub fn send_command(conn: &mut MdswpStream, command: c2s::Command) -> io::Result<()> {
    // Convert to JSON and encrypt:
    let json = serde_json::to_string(&command).unwrap();
    let bytes = json.into_bytes();
    let encrypted = encrypt(bytes);
    // Check length:
    let len = encrypted.len();
    if len > u32::MAX as usize {
        return Result::Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Maximum size of a single command exceeded."
        ))
    }
    // Send command:
    let len = len as u32;
    conn.write_all(&len.to_be_bytes())?;
    conn.write_all(&encrypted)?;
    conn.flush()?;
    Result::Ok(())
}

/// Method for receiving single command from the server.
pub fn recv_command(conn: &mut MdswpStream) -> io::Result<s2c::Command> {
    // Functions used in closures:
    fn cannot_decode<E>(err: E) -> io::Error where E: Error {
        io::Error::new(io::ErrorKind::BrokenPipe,
            format!("Could not decode received command: {}", err))
    }
    // Load length
    let mut len = [0; 4];
    conn.read_exact(&mut len)?;
    let len = u32::from_be_bytes(len) as usize;
    // Load encrypted content:
    let mut buf = vec![0; len];
    conn.read_exact(&mut buf)?;
    // Decrypt and decode:
    let decrypted = decrypt(buf);
    let string = String::from_utf8(decrypted).map_err(cannot_decode)?;
    let command = serde_json::from_str(&string).map_err(cannot_decode)?;
    // Return Ok if successful:
    Result::Ok(command)
}

#[doc(hidden)]
fn encrypt(data: Vec<u8>) -> Vec<u8> {
    data
}

#[doc(hidden)]
fn decrypt(data: Vec<u8>) -> Vec<u8> {
    data
}

pub fn error(conn: &mut MdswpStream, message: &str) -> ! {
    let _ = conn.reset();
    println!("FATAL: {}", message);
    exit(1)
}