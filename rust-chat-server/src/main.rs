//! Basic TCP server

use std::net::{
    TcpListener,
    TcpStream,
};

use std::thread::spawn;
use std::io::Write;

/// Handles received TCP requests
///
/// TODO: define the function
///
/// # Arguments:
///
/// * `stream` - TCP stream between the server and the new connected client
fn handle_request(mut stream: TcpStream) {

    stream.write("Welcome to rust-chat-server".as_bytes()).unwrap();
}

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    for income in listener.incoming() {

        match income {
            Ok(stream) => {

                let client_address = stream.peer_addr().unwrap();
                println!(
                    "New client connected: {}",
                    client_address,
                );

                spawn(|| {
                    handle_request(stream);
                });
            }
            Err(_) => {
                println!("Client connection failed.");
            }
        }
    }
}
