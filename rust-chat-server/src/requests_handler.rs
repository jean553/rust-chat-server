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
    /* create a buffer in order to read data sent through the stream;
       in other words, the data sent by the client attached to this stream */
    let mut buffer = BufReader::new(stream);

    let mut message = String::new();

    loop {

        /* blocking step to read data from the client stream */
        let request = buffer.read_line(&mut message);

        if request.is_err() {
            continue;
        }

        /* get message as bytes slice in order to check
           what character exactly has been sent */
        let message_copy = message.clone();
        let message_bytes = message_copy.as_bytes();

        /* ignore the message if the first character
           is a carriage return */
        const END_OF_LINE: u8 = 10;
        if message_bytes.get(0) == Some(&END_OF_LINE) {
            break;
        }

        let send_message = sender.send(message.to_string());
        if send_message.is_err() {
            break;
        }

        message.clear();
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
