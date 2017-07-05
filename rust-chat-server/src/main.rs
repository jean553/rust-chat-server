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
fn handle_request(stream: TcpStream) {
    println!("New client connected.");
}

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    for income in listener.incoming() {

        match income {
            Ok(stream) => {
                handle_request(stream);
            }
            Err(_) => {
                println!("Client connection failed.");
            }
        }
    }
}
