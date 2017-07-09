//! Basic TCP server

use std::net::{
    TcpListener,
    TcpStream,
};

use std::thread::spawn;
use std::io::{
    Write,
    BufReader,
    BufRead,
};

use std::sync::mpsc;

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
    let mut message = String::new();

    loop {

        let request = buffer.read_line(&mut message); // blocking IO

        match request {
            Ok(_) => {

                let message_bytes = message.clone();
                let bytes = message_bytes.as_bytes();

                const END_OF_LINE: u8 = 10;
                match bytes.get(0) {
                    Some(&END_OF_LINE) | None => {
                        break;
                    },
                    Some(&_) => {
                        println!(
                            "Client {} sent message: {}",
                            client_id,
                            message,
                        );

                        message.clear();
                    }
                };
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

    let (sender, receiver): (
        mpsc::Sender<String>,
        mpsc::Receiver<String>
    ) = mpsc::channel();

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
