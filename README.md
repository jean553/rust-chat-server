[![Build Status](https://travis-ci.org/jean553/rust-chat-server.svg?branch=master)](https://travis-ci.org/jean553/rust-chat-server)

# rust-chat-server

## Installation

```bash
vagrant up
```

## Connection

```bash
vagrant ssh
```

## Execution

```bash
cargo run
```

## Generate documentation

```
cargo rustdoc -- --no-defaults
```

## Send messages from your host

```bash
nc localhost 9090
```

## How it works ?

### Architecture

```
+---------+     +--------------------+                                          +-----------------------+
|  Client +----->  Thread and sender +--+                                  +---->  Thread and receiver  |
+---------+     +--------------------+  |                                  |    +-----------------------+
                                        |    +---------------------------+ |
                                        |    |  Unique thread to listen  | |
                                        |    |     incoming messages     | |
+---------+     +--------------------+  |    |                           | |    +-----------------------+
|  Client +----->  Thread and sender +------->                           +------>  Thread and receiver  |
+---------+     +--------------------+  |    |         Receiver          | |    +-----------------------+
                                        |    |                           | |
                                        |    |      Dynamic array        | |
                                        |    |        of senders         | |
+---------+     +--------------------+  |    |                           | |    +-----------------------+
|  Client +----->  Thread and sender +--+    +---------------------------+ +---->  Thread and receiver  |
+---------+     +--------------------+                                          +-----------------------+
```

### 1 - Create the TCP listener

The TCP listener opens a socket for a given address.
It listens for TCP connections.

```rust
let listener = TcpListener::bind("0.0.0.0:9090").unwrap();
```

We use `unwrap` as we want the program to stop if the TCP socket cannot be opened.

### 2 - Create a dynamic array to store all the client senders

Everytime a new client connects to the server, a new thread is started.
The thread handles all the messages sent to the server from the client.
Every thread communicates to a global thread through individual senders
that all send data to an unique thread.

```
  +------------------------------------------------------+
  |                                                      |
  |    Server         +--------------+                   |
  |                   |              |                   |
  |                   |Unique thread |                   |
  |                   |              |                   |
  |                   |   Receiver   |                   |
  |                   |              |                   |
  |                   +------^-------+                   |
  |                          |                           |
  |        +-----------------------------------+         |
  |        |                 |                 |         |
  |        |                 |                 |         |
  |  +-----+------+    +-----+------+    +-----+------+  |
  |  |   Sender   |    |   Sender   |    |   Sender   |  |
  |  |            |    |            |    |            |  |
  |  |   Thread   |    |   Thread   |    |   Thread   |  |
  |  +-----^------+    +-----^------+    +-----^------+  |
  |        |                 |                 |         |
  |        |                 |                 |         |
  +------------------------------------------------------+
           |                 |                 |
           |                 |                 |
     +-----+------+    +-----+------+    +-----+------+
     |   Client   |    |   Client   |    |   Client   |
     +------------+    +------------+    +------------+

```

First, we create a sender/receiver pair. The sender can be cloned.
The receiver has to be unique.

```rust
let (sender, receiver): (
    mpsc::Sender<String>,
    mpsc::Receiver<String>
) = mpsc::channel();
```

We then create the dynamic array to store all the senders we will create.

```rust
type Senders = Vec<mpsc::Sender<String>>;
let senders: Senders = Vec::new();
```

### 3 - Make our senders availables for all threads

Each time a new user connect, we create a new sender for this user
and store the sender into a dynamic array of senders.
This is what our main thread does: the thread that handles incoming connections.

There is a second thread that listen for incoming messages from all the clients
threads and forward the messages to all the senders (in order to broadcast them
to everybody).
This is what the second thread does: send messages to every senders
of the dynamic array threads.

Two threads cannot access a `Vec<T>` at the same time (for concurrency safety reasons).
This restriction is ensured by the `std::thread::spawn` method. It is not possible
to pass a reference to a `Vec<T>` through the closure, so it ensures two threads
will never try to access to the same data at the same moment.

One way to pass the same `Vec<T>` to multiple threads is to wrap it
into an atomic references counter (Arc), locking and unlocking the vector
into each thread using a mutex.

```rust
type Senders = Vec<mpsc::Sender<String>>;

let mutex: Mutex<Senders> = Mutex::new(senders);
let first_senders_list: Arc<Mutex<Senders>> = Arc::new(mutex);
let second_senders_list = first_senders_list.clone();
```

By this way, cloning the `Arc<T>` object creates a second reference
to the same array. The two references can now access the array
from two different threads safely (concurrent access is secured).

### 4 - Create the unique thread that forward messages from any client to all senders

An unique thread listens for any message received from any client
and forwards it to all the senders that have been created (array of senders).

```rust
pub fn receive_messages(
    receiver: mpsc::Receiver<String>,
    senders_arc: Arc<Mutex<Vec<mpsc::Sender<String>>>>,
) {

    loop {
        let message = receiver.recv(); // blocking

        match message {
            Ok(value) => {

                /* lock the senders array for concurrent access */
                let guard = senders_arc.lock().unwrap();
                let senders = &*guard;

                /* send the message to all the senders */
                for sender in senders {
                    sender.send(value.to_string()).expect("cannot send messages");
                }
            }
            Err(_) => {
            }
        }
    }
}

...

/* start one thread */
spawn(|| {
    requests_handler::receive_messages(
        receiver,
        first_senders_list,
    );
});
```
