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

### 1 - Creates the TCP listener

The TCP listener opens a socket for a given address.
It listens for TCP connections.

```rust
let listener = TcpListener::bind("0.0.0.0:9090").unwrap();
```

We use `unwrap` as we want the program to stop if the TCP socket cannot be opened.
