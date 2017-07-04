use std::net::{TcpListener};

fn main() {

    let listener = TcpListener::bind("0.0.0.0:9090").unwrap();

    for income in listener.incoming() {

        match income {
            Ok(_) => {
                println!("New client connected.");
            }
            Err(_) => {
                println!("Client connection failed.");
            }
        }
    }
}
