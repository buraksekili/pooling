use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use thread_pool_rs::ThreadPool;

// Function to handle a client connection
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    thread::sleep(Duration::from_millis(100)); // Simulate some work
    stream.write(&buffer).unwrap();
}

// Server using thread pool
fn run_pooled_server() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(50);
    println!("Running TCP with pooling through :7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_client(stream);
            Ok(())
        });
    }
}

// Server spawning a new thread for each connection
fn run_spawning_server() {
    let listener = TcpListener::bind("127.0.0.1:7879").unwrap();
    println!("Running TCP without pooling through :7879");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_client(stream);
        });
    }
}

fn main() {
    // Run either server based on command-line argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "pool" {
        run_pooled_server();
    } else {
        run_spawning_server();
    }
}
