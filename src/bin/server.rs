use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use thread_pool_rs::ThreadPool;

// Function to handle a client connection
fn handle_client(mut stream: TcpStream) {
    // println!("Handling client connection");
    stream
        .set_nonblocking(true)
        .expect("Failed to set non-blocking");

    let mut buffer = [0; 1024];
    let mut response = String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!");

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                if let Ok(_) = stream.write_all(response.as_bytes()) {}
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available yet, yield to other tasks
                thread::yield_now();
                continue;
            }
            Err(e) => {
                break;
            }
            _ => {}
        }
    }
    // println!("Finished handling client connection");
}

// Server using thread pool
fn run_pooled_server(pool_size: usize) {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to address");
    listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    let pool = Arc::new(ThreadPool::new(pool_size));
    println!(
        "Running TCP with pooling (size: {}) through :7878",
        pool_size
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool = Arc::clone(&pool);
                if let Err(e) = pool.execute(move || {
                    handle_client(stream);
                    Ok(())
                }) {
                    // eprintln!("Failed to execute job: {}", e);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection available, yield to other tasks
                thread::yield_now();
                continue;
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

// Server spawning a new thread for each connection
fn run_spawning_server() {
    let listener = TcpListener::bind("127.0.0.1:7879").expect("Failed to bind to address");
    println!("Running TCP without pooling through :7879");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "pool" {
        let pool_size = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
        run_pooled_server(pool_size);
    } else {
        run_spawning_server();
    }
}
