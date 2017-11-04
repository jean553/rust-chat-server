//! Methods that handle clients requests

use std::net::TcpStream;

use std::io::{
    Write,
    BufReader,
    BufRead,
};

use std::sync::{
    mpsc,
    Mutex,
    Arc,
};

/// Shares a received message to all threads
///
/// # Arguments:
///
/// * `message` - the message posted by the client
/// * `sender` - channel sender for communication between threads
fn share_message(
    message: &mut String,
    sender: &mpsc::Sender<String>,
) -> bool {

    let message_to_send = format!(
        "Client sent message: {}",
        message,
    );

    match sender.send(message_to_send.to_string()) {
        Ok(_) => {
            message.clear();
        },
        Err(_) => {
            println!("Error: cannot read message from client");
            return false;
        }
    };

    true
}

/// Handles received TCP requests
///
/// # Arguments:
///
/// * `stream` - TCP stream between the server and the new connected client
/// * `sender` - the sender to uses to forward the received message
pub fn handle_request(
    stream: TcpStream,
    sender: mpsc::Sender<String>,
) {
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

                        let shared = share_message(
                            &mut message,
                            &sender,
                        );

                        if !shared {
                            break;
                        }
                    }
                };
            }
            _ => {

                println!("Error: cannot read request from client");

                break;
            }
        }
    }
}

/// Run by an unique thread started at the launch of the program.
/// Continuously listens for connections from the receiver
/// and forward it to every senders of the senders list (one per client)
///
/// # Arguments:
///
/// * `receiver` - Channel receiver to get messages
/// * `senders_mutex_pointer` - Atomic reference-counting pointer for the senders array
pub fn receive_messages(
    receiver: mpsc::Receiver<String>,
    senders_mutex_pointer: Arc<Mutex<Vec<mpsc::Sender<String>>>>,
) {

    loop {

        /* blocking listening procedure for incoming messages */
        let message_result = receiver.recv();

        /* ignore the message if this is an error result object */
        if message_result.is_err() {
            continue;
        }

        /* acquires the senders mutex, blocks until it is available */
        let guard = senders_mutex_pointer.lock().unwrap();

        /* create a reference to the senders list, first access it through
           the pointer and then creates a reference to this array */
        let senders = &*guard;

        /* get the message from the receiver result */
        let message = message_result.unwrap();

        /* send the message to every senders into the senders array
           (send the message to every client) */
        for sender in senders {
            sender.send(message.to_string())
                .expect("cannot send message");
        }
    }
}

/// Sends received messages to all the clients
///
/// # Arguments:
///
/// * `stream` - TCP stream between the server and the new connected client
/// * `receiver` - Channel receiver to get messages
pub fn send_to_client(
    mut stream: TcpStream,
    receiver: mpsc::Receiver<String>,
) {

    loop {
        let message = receiver.recv(); // blocking IO

        match message {
            Ok(value) => {
                stream.write(value.as_bytes()).unwrap();
            }
            Err(_) => {
            }
        }

    }
}
