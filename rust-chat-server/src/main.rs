//! Basic TCP server

use std::net::TcpListener;
use std::thread::spawn;
use std::sync::{
    Mutex,
    Arc,
};
use std::sync::mpsc::{
    Sender,
    Receiver,
    channel,
};

mod requests_handler;

fn main() {

    /* create a TCP socket server, listening for connections */
    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    /* creation of a channel for messages transmission,
       one sender can send messages to one receiver */
    let (sender, receiver): (
        Sender<String>,
        Receiver<String>
    ) = channel();

    /* TODO: explanation */
    type Senders = Vec<Sender<String>>;
    let senders: Senders = Vec::new();
    let senders_mutex: Mutex<Senders> = Mutex::new(senders);
    let senders_list: Arc<Mutex<Senders>> = Arc::new(senders_mutex);

    /* TODO: explanation */
    let senders_list_copy = senders_list.clone();

    spawn(|| {
        requests_handler::receive_messages(
            receiver,
            senders_list,
        );
    });

    let mut clients_count = 0;

    for income in listener.incoming() {

        match income {
            Ok(stream) => {

                let client_address = stream.peer_addr().unwrap();
                println!(
                    "New client connected: {}",
                    client_address,
                );

                let sender = sender.clone();

                let other_stream = stream.try_clone()
                    .expect("Cannot clone TCP stream");

                spawn(move || {
                    requests_handler::handle_request(
                        stream,
                        clients_count,
                        sender,
                    );
                });

                let (
                    writer_sender,
                    writer_receiver
                ): (
                    Sender<String>,
                    Receiver<String>
                ) = channel();

                let mut guard = senders_list_copy.lock().unwrap();
                let mut senders = &mut *guard;
                senders.push(writer_sender);

                spawn(|| {
                    requests_handler::send_to_client(
                        other_stream,
                        writer_receiver,
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
