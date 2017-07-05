//! Basic TCP server

use std::net::{
    TcpListener,
    TcpStream,
};

use std::thread::spawn;
use std::io::{
    Write,
    BufReader,
    Read,
};

/// Handles received TCP requests
///
/// TODO: define the function
///
/// # Arguments:
///
/// * `stream` - TCP stream between the server and the new connected client
/// * `client_id` - unique id of the handled client
fn handle_request(
    mut stream: TcpStream,
    client_id: u8,
) {
    stream.write("Welcome to rust-chat-server\n".as_bytes()).unwrap();

    let mut buffer = BufReader::new(stream);

    const BYTES_PER_MESSAGE: usize = 20;
    let mut bytes: [u8; BYTES_PER_MESSAGE] = [0; BYTES_PER_MESSAGE];

    loop {

        let request = buffer.read(&mut bytes); // blocking IO

        match request {
            Ok(_) => {

                const NEXT_LINE_BYTE: u8 = 10;
                if bytes[0] == NEXT_LINE_BYTE {
                    break;
                }

                println!(
                    "Client {} sent message:",
                    client_id,
                );

                for &byte in bytes.iter() {
                    println!("{}", byte);
                }

                bytes = [0; BYTES_PER_MESSAGE];
            }
            _ => {

                println!(
                    "Error: cannot read message from client {}",
                    client_id,
                );

                break;
            }
        }
    }
}

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    let mut clients_count: u8 = 0;

    for income in listener.incoming() {

        match income {
            Ok(stream) => {

                let client_address = stream.peer_addr().unwrap();
                println!(
                    "New client connected: {}",
                    client_address,
                );

                spawn(move || {
                    handle_request(
                        stream,
                        clients_count,
                    );
                });

                clients_count += 1;
            }
            Err(_) => {
                println!("Error: one client could not connect.");
            }
        }
    }
}
