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

#[macro_use]
mod macros;

mod client;
mod client_list;
mod listener;
mod message_list;
mod message_queue;
mod user;
mod user_list;

use mdchat_serverconf::Config;

use mdlog::Logger;
use mdlog::LogLevel;

use mdswp::MdswpListener;

use once_cell::sync::OnceCell;

use std::process::exit;
use std::sync::Arc;
use std::thread;

static GLOBAL_CONFIG: OnceCell<Arc<Config>> = OnceCell::new();

/// Returns reference with interior mutability to the global configuration, that is
/// an [`Arc`] pointing to a server-global [`Config`] instance.
fn global_config() -> Arc<Config> {
    GLOBAL_CONFIG.get().unwrap().clone()
}

/// Loads global configuration file. If there is an error, the program ends with
/// exit code 1.
fn load_global_config() {
    let config = Config::default();
    match config.process_file("/etc/mdchat-server.conf", false) {
        Result::Err(err) => {
            eprintln!("Could not load configuration file:\n{:?}", err);
            exit(1);
        },
        Result::Ok(()) => GLOBAL_CONFIG.set(Arc::new(config))
            .map_err(|_| panic!("Value already set")).unwrap(),
    }
}

/// Encrypts data as given by global configuration.
///
/// # Parameters
///
///  -  `data`: data to be encrypted
///
/// # Return value
///
/// A [`Vec`] of bytes ([`u8`]) representing the encrypted data.
fn encrypt(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// Decrypts data as given by global configuration.
///
/// # Parameters
///
///  -  `data`: data to be decrypted
///
/// # Return value
///
/// A [`Vec`] of bytes ([`u8`]) representing the decrypted data.
fn decrypt(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// Logs a message using logger configured by global configuration.
fn log(log_level: LogLevel, message: &str) {
    global_config().logger().write().unwrap().log(log_level, message).unwrap();
}

fn main() {
    // Load config
    load_global_config();
    log(LogLevel::Info, "Configuration file loaded successfully");

    // Initialize listeners for incoming connections:
    let global_config = global_config();
    let listen_sock_addrs = global_config.listen_sock_addrs().read().unwrap();
    let mut listener_threads = Vec::with_capacity(listen_sock_addrs.len());
    for sock_addr in &*listen_sock_addrs {
        match MdswpListener::bind(sock_addr) {
            Result::Err(err) => {
                let message = format!("Could not bind to {}: {}", sock_addr, err);
                log(LogLevel::Error, &message);
            },
            Result::Ok(listener) => {
                let thread = thread::spawn(|| listener::listen(listener));
                let message =  format!("Listening at {}", sock_addr);
                log(LogLevel::Info, &message);
                listener_threads.push(thread);
            }
        }
    }

    // No listener means server cannot run.
    if listener_threads.is_empty() {
        log(LogLevel::Fatal, "There is no socket to listen for incoming connections. Quitting.");
        exit(2);
    }

    // Message handler:
    let message_handler = thread::Builder::new()
        .name("Message handler".to_string())
        .spawn(message_queue::handle_incoming)
        .unwrap();
    message_handler.join().unwrap().unwrap();
}
