//! Basic TCP server

mod requests_handler;

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

use requests_handler::{
    receive_messages,
    handle_sent_messages,
    send_to_client,
};

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
    let senders_mutex_pointer: Arc<Mutex<Senders>> = Arc::new(senders_mutex);

    /* copy the senders mutex pointer as we move it
       right after when creating the listening thread
       and we still want to be able to access it
       from the main thread */
    let senders_mutex_pointer_copy = senders_mutex_pointer.clone();

    /* create a thread that listens for all incoming messages
       and forward them to every connected clients */
    spawn(|| {
        receive_messages(
            receiver,
            senders_mutex_pointer,
        );
    });

    /* listener.incoming() returns an iterator to the TCP listeners clients;
       the loop content is executed everytime a new client connects to the server;
       the next() method returns Option<Result<TcpStream>>, so income is a Result<TcpStream> */
    for income in listener.incoming() {

        /* silently ignore any error if the client connection failed */
        if income.is_err() {
            continue;
        }

        /* get the TCP stream object from the connection
           in order to use the connected client */
        let stream = income.unwrap();

        /* get the address and port of the remove peer of the given client */
        let client_address = stream.peer_addr()
            .unwrap();

        println!(
            "New client connected: {}",
            client_address,
        );

        /* the new client stream is copied
           as it is passed to two different threads */
        let stream_copy = stream.try_clone()
            .expect("Cannot clone TCP stream");

        /* create a new sender from the channel sender;
           there is one sender per client;
           this new sender is also part of the unique receiver channel */
        let sender_copy = sender.clone();

        /* create a thread that handles sent messages from the new client */
        spawn(|| {
            handle_sent_messages(
                stream_copy,
                sender_copy,
            );
        });

        /* create one new sender/receiver couple by client;
           the sender gets the data from the global receiver
           and sends it to each client dedicated receiver */
        let (
            client_sender,
            client_receiver
        ): (
            Sender<String>,
            Receiver<String>
        ) = channel();

        /* create one thread per client that has one receiver per sender
           into the senders dynamic array; every thread takes the value
           from the receiver and inserts it into the client stream */
        spawn(|| {
            send_to_client(
                stream,
                client_receiver,
            );
        });

        /* acquires the senders mutex, blocks until it is available */
        let mut guard = senders_mutex_pointer_copy.lock().unwrap();

        /* create a reference to the senders list, first access it through
           the pointer and then creates a reference to this array */
        let mut senders = &mut *guard;

        /* the dedicated client sender is added to the list of senders 
           used by the global receiver to forward messages */
        senders.push(client_sender);
    }
}
