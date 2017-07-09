//! Basic TCP server

use std::net::TcpListener;
use std::thread::spawn;
use std::sync::mpsc;

mod requests_handler;

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    let mut clients_count: u8 = 0;

    let (sender, receiver): (
        mpsc::Sender<String>,
        mpsc::Receiver<String>
    ) = mpsc::channel();

    spawn(move || {
        requests_handler::receive_messages(&receiver);
    });

    for income in listener.incoming() {

        match income {
            Ok(stream) => {

                let client_address = stream.peer_addr().unwrap();
                println!(
                    "New client connected: {}",
                    client_address,
                );

                let sender = sender.clone();

                spawn(move || {
                    requests_handler::handle_request(
                        stream,
                        clients_count,
                        sender,
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
