mod client;
mod client_list;
mod listener;
mod message_list;
mod message_queue;
mod user;
mod user_list;

use mdchat_serverconf::Config;

use mdlog::CompositeLogger;
use mdlog::Logger;

use mdswp::MdswpListener;

use once_cell::sync::OnceCell;

use std::process::exit;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

static GLOBAL_CONFIG: OnceCell<Arc<Config>> = OnceCell::new();
static GLOBAL_LOGGER: OnceCell<Arc<CompositeLogger>> = OnceCell::new();

/// Returns reference with interior mutability to the global configuration, that is
/// an [`Arc`] pointing to a [`RwLock`] containing server-global [`Config`]
/// instance.
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
    let _ = GLOBAL_CONFIG.set(Arc::new(config));
}

/// Here the server starts.
fn main() {
    // Load config
    load_global_config();
    let global_config = get_global_config();

    // Initialize listeners for incoming connections:
    let listen_sock_addrs = global_config.listen_sock_addrs();
    let mut listener_threads = Vec::with_capacity(listen_sock_addrs.len());
    for sock_addr in listen_sock_addrs {
        match MdswpListener::bind(sock_addr) {
            Result::Err(err) => {

            },
            Result::Ok(listener) => {
                let thread = thread::spawn(|| listener::listen(listener));
                listener_threads.push(thread);
            }
        }
    }

    let message_handler = thread::spawn(message_queue::handle_incoming);
    let _ = message_handler.join().unwrap();
}
