use crate::client;
use crate::client_list;

use std::net::SocketAddr;
use std::thread;

use mdswp::MdswpListener;
use mdswp::MdswpStream;

/// Method for infinite accepting a connection. This is a blocking method to be run
/// in a separate thread.
///
/// # Parameters:
///
///  -  `listener`: the listener to listen on
pub fn listen(listener: MdswpListener) {
    for client in listener.incoming() {
        match client {
            Result::Ok((stream, peer_addr)) => __handle_conn(stream, peer_addr),
            Result::Err(err) => todo!() //logger.log(Local::now(), LogLevel::Warning,
                //&format!("A client could not connect to the server: {}", err))
        }
    }
}

#[doc(hidden)]
fn __handle_conn(stream: MdswpStream, peer_addr: SocketAddr) {
    let stream_clone = stream.try_clone().unwrap();
    // Run a thread for the client
    let handle = thread::Builder::new()
        .name(format!("Client thread for {}", peer_addr))
        .spawn(move || client::client_thread(stream_clone, peer_addr))
        .unwrap();
    // Add new client stream to the clients:
    client_list::add_connection(stream, handle);
}