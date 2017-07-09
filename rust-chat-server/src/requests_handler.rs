//! Methods that handle clients requests

use std::net::TcpStream;

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
pub fn handle_request(
    mut stream: TcpStream,
    client_id: u8,
    sender: mpsc::Sender<String>,
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

                        let message_to_send = format!(
                            "Client {} sent message: {}",
                            client_id,
                            message,
                        );

                        match sender.send(message_to_send.to_string()) {
                            Ok(_) => {
                                message.clear();
                            },
                            Err(_) => {
                                println!(
                                    "Error: cannot read message from client {}",
                                    client_id,
                                );

                                break;
                            }
                        };
                    }
                };
            }
            _ => {

                println!(
                    "Error: cannot read request from client {}",
                    client_id,
                );

                break;
            }
        }
    }
}

/// Started by an independant thread that listens
/// for new messages and sends them to every thread
///
/// # Arguments:
///
/// * `receiver` - Channel receiver to get messages
pub fn receive_messages(receiver: &mpsc::Receiver<String>) {

    loop {
        let message = receiver.recv(); // blocking IO

        match message {
            Ok(value) => {
                println!("Received message: {}", value);
            }
            Err(_) => {
            }
        }
    }
}

