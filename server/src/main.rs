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
fn get_global_config() -> Arc<Config> {
    GLOBAL_CONFIG.get().unwrap().clone()
}

/// Loads global configuration file. If there is an error, the program ends with
/// exit code 1.
fn load_global_config() {
    let mut config = Config::default();
    if let Result::Err(err) = config.process_file("/etc/mdchat-server.conf", false) {
        eprintln!("Could not load configuration file:");
        eprintln!("{:?}", err);
        exit(1);
    }
    GLOBAL_CONFIG.set(Arc::new(config))
        .map_err(|err| panic!("Value already set"));
}

fn log(log_level: LogLevel, message: &str) {
    get_global_config().logger().write().unwrap().log(log_level, message);
}

/// Here the server starts.
fn main() {
    // Load config
    load_global_config();
    let global_config = get_global_config();

    // Initialize listeners for incoming connections:
    let listen_sock_addrs = global_config.listen_sock_addrs().read().unwrap();
    let mut listener_threads = Vec::with_capacity(listen_sock_addrs.len());
    for sock_addr in &*listen_sock_addrs {
        match MdswpListener::bind(sock_addr) {
            Result::Err(err) => {
                let message = format!("Could not bind to {}: {}", sock_addr, err);
                log(LogLevel::Warning, &message);
            },
            Result::Ok(listener) => {
                let thread = thread::spawn(|| listener::listen(listener));
                let message =  format!("Listening at {}", sock_addr);
                log(LogLevel::Info, &message);
                listener_threads.push(thread);
            }
        }
    }

    if listener_threads.is_empty() {
        log(LogLevel::Fatal, "There is no socket to listen for incoming connections. Quitting.");
        exit(2);
    }

    let message_handler = thread::spawn(message_queue::handle_incoming);
    let _ = message_handler.join().unwrap();
}
