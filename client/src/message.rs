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

use std::error::Error;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process::exit;

use mdcrypt::Decrypt;
use mdcrypt::TryEncrypt;

use mdlog::Logger;
use mdlog::LogLevel;

pub fn handle_incoming<R>(mut reader: R)
where
    R: Read,
{
    loop {
        if let Result::Err(err) = listen_for(&mut reader)
            .and_then(|recvd| handle_message(recvd))
        {
            handle_err(err);
            return;
        }
    }
}

pub fn handle_message(bytes: Vec<u8>) -> io::Result<()> {
    let decrypted: Vec<u8> = MESSAGE_CRYPT.decrypt(bytes);
    let string = match String::from_utf8(decrypted) {
        Ok(s) => s,
        Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
    };
    log(LogKind::Info, string);
    return Ok(());
}

pub fn handle_err<E>(error: E)
where
    E: Error,
{
    log(LogKind::Error, format!("Error: {:?}. Quitting.", error));
    exit(2)
}

pub fn listen_for<R>(reader: &mut R) -> io::Result<Vec<u8>>
where
    R: Read,
{
    let mut buffer = [0; 2];
    reader.read_exact(&mut buffer)?;
    let data_len = u16::from_be_bytes(buffer) as usize;

    let mut buffer = Vec::with_capacity(data_len);
    for _ in 0..data_len {
        buffer.push(0);
    }
    reader.read_exact(&mut buffer[0..data_len])?;
    Ok(buffer)
}

pub fn send<W, E>(writer: &mut W, encrypter: &E, message: &str) -> io::Result<()>
where
    W: Write,
    E: TryEncrypt,
{
    let bytes = message.trim().as_bytes();

    let encrypted_bytes: Vec<u8> = match encrypter.try_encrypt(bytes.to_vec()) {
        Ok(byte_vec) => byte_vec,
        Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidInput, err.to_string())),
    };

    if encrypted_bytes.len() > u16::MAX.into() {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Data too long"))
    } else {
        let len = encrypted_bytes.len() as u16;
        writer.write(&len.to_be_bytes())?;
        writer.write(&encrypted_bytes)?;
        writer.flush()?;
        Ok(())
    }
}