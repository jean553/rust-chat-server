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

    /* create a dynamic array of senders for string messages,
       one sender per client */
    type Senders = Vec<Sender<String>>;
    let senders: Senders = Vec::new();

    /* there is one global receiver that listen for messages from any client
       and there is a dynamic list of senders for every client,
       in order to forward messages to each client;
       the senders list is part of the main thread in order to create a new sender
       everytime a new client connects and it is also part of the thread
       that receive and forward messages to each client;
       the senders dynamic array is shared between threads, in order to prevent
       concurrent access, we protect it into a mutex */
    let senders_mutex: Mutex<Senders> = Mutex::new(senders);

    /* the senders array is shared between threads; in order to access it 
       from multiple threads, we simply put the mutex into an atomically
       reference counted pointer; Arc<T> provides thread-safe shared ownership
       of the passed data; it can be copied through threads and always point
       to the same heap memory */
    let senders_list: Arc<Mutex<Senders>> = Arc::new(senders_mutex);

    /* TODO: explanation */
    let senders_list_copy = senders_list.clone();

    /* create a thread that listens for all incoming messages
       and forward them to every connected clients */
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
