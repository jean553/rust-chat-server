//! Basic TCP server

use std::net::TcpListener;
use std::thread::spawn;
use std::sync::{
    mpsc,
    Mutex,
    Arc,
};

mod requests_handler;

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    let mut clients_count: u8 = 0;

    let (sender, receiver): (
        mpsc::Sender<String>,
        mpsc::Receiver<String>
    ) = mpsc::channel();

    let mut senders: Vec<mpsc::Sender<String>> = Vec::new();
    let mutex: Mutex<Vec<mpsc::Sender<String>>> = Mutex::new(senders);
    let arc_senders: Arc<Mutex<Vec<mpsc::Sender<String>>>> = Arc::new(mutex);

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
