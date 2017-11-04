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

/// Run by threads created at each client connection. Handles the sent messages by one client. 
/// There is one thread per client.
///
/// # Arguments:
///
/// * `stream` - TCP stream between the server and the new connected client
/// * `sender` - the sender to use to forward the received message
pub fn handle_sent_messages(
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

        /* attempt to send the message through the current client sender;
           as the sender is part of the senders dynamic array
           created from one unique receiver, the messages sent by every client
           all go to this unique receiver for forward */
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

/// Run by threads created at each client connection.
/// Each thread has a receiver that waits for data
/// send by the client sender from the senders dynamic array;
/// the thread forward the message into the client dedicated stream
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

        /* the client receiver listens for messages,
           this is a blocking IO */
        let message_result = receiver.recv();

        if message_result.is_err() {
            continue;
        }

        /* the message is pushed into the dedicated client stream
           so the client can receive it */
        let message = message_result.unwrap();
        let message_bytes = message.as_bytes();
        stream.write(message_bytes).unwrap();
    }
}
