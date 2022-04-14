mod message;

use std::io::stdin;
use std::io::stdout;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::process::exit;
use std::thread;

use mdlog::LogLevel;

use mdswp::MdswpStream;

/// Main entry for MDChat client
fn main() {
    let mut stdin_reader = BufReader::new(stdin());
    let mut stdin_buffer = String::new();

    let mut ip_addr: Option<IpAddr> = None;
    let mut port: Option<u16> = None;

    while matches!(ip_addr, None) {
        print!("Server IP address: ");
        stdout().flush().unwrap();
        stdin_reader.read_line(&mut stdin_buffer).unwrap();
        match stdin_buffer.trim().parse() {
            Ok(ip) => ip_addr = Some(ip),
            Err(err) => log(LogLevel::Error, err),
        }
        stdin_buffer.clear();
    }

    while matches!(port, None) {
        print!("Port: ");
        stdout().flush().unwrap();
        stdin_reader.read_line(&mut stdin_buffer).unwrap();
        match stdin_buffer.trim().parse() {
            Ok(p) if p >= 1000 => port = Some(p),
            Ok(p) => log(
                LogLevel::Error,
                format!("Invalid input: Port number {} too small", p),
            ),
            Err(err) => log(LogLevel::Error, err),
        }
        stdin_buffer.clear();
    }

    let ip_addr = ip_addr.unwrap();
    let port = port.unwrap();
    let socket = SocketAddr::new(ip_addr, port);

    let mut username = String::new();
    print!("Username: ");
    stdout().flush().unwrap();
    stdin_reader.read_line(&mut username).unwrap();

    let mut password = String::new();
    print!("Password: ");
    stdout().flush().unwrap();
    stdin_reader.read_line(&mut password).unwrap();

    let tcp_conn = match MdswpStream::connect(socket) {
        Ok(stream) => stream,
        Err(err) => {
            log(LogLevel::Fatal, format!("Could not connect: {}", err));
            exit(1)
        }
    };

    log(
        LogLevel::Info,
        "Connected to server successfully. Now you can type your messages",
    );

    let mut tcp_conn_writer = BufWriter::new(tcp_conn.try_clone().unwrap());
    thread::spawn(|| message::handle_incoming(tcp_conn));

    let login_message = username.trim().to_string() + "\n" + password.trim();
    match message::send(&mut tcp_conn_writer, &*MESSAGE_CRYPT, &login_message) {
        Ok(_) => {}
        Err(err) => {
            log(LogLevel::Fatal, format!("Error: {}", err));
            exit(1)
        }
    }

    loop {
        let mut stdin_buffer = String::new();
        stdin_reader.read_line(&mut stdin_buffer).unwrap();
        let trim = stdin_buffer.trim();
        match message::send(&mut tcp_conn_writer, &*MESSAGE_CRYPT, trim) {
            Ok(_) => {}
            Err(err) => {
                log(LogLevel::Fatal, format!("Error: {}", err));
                exit(1)
            }
        }
    }
}