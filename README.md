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

TODO: add schema

### 1 - Create the TCP listener

The TCP listener opens a socket for a given address.
It listens for TCP connections.

```rust
let listener = TcpListener::bind("0.0.0.0:9090").unwrap();
```

We use `unwrap` as we want the program to stop if the TCP socket cannot be opened.

### 2 - Creates a dynamic array to store all the client senders

Everytime a new client connects to the server, a new thread is started.
The thread handles all the messages sent to the server from the client.
Every thread communicates to a global thread through individual senders
that all send data to an unique thread.

```
  +------------------------------------------------------+
  |                                                      |
  |    Server         +--------------+                   |
  |                   |              |                   |
  |                   | Unique thrad |                   |
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
