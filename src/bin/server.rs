use rand::Rng;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use thread_pool_rs::ThreadPool;

fn sleep_random_duration(min_ms: u64, max_ms: u64) {
    let mut rng = rand::thread_rng();
    let sleep_duration = rng.gen_range(min_ms..=max_ms);

    thread::sleep(Duration::from_millis(sleep_duration));
}

// Function to handle a client connection
fn handle_client(mut stream: TcpStream) {
    sleep_random_duration(84, 218);

    let mut buffer = [0; 1024];
    let response = String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!");

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
            _ => {}
        }
    }
    // println!("Finished handling client connection");
}

// Server using thread pool
fn run_pooled_server(pool_size: usize) {
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Failed to bind to address");
    let pool = Arc::new(ThreadPool::new(pool_size));
    println!(
        "Running TCP with pooling (size: {}) through :7878",
        pool_size
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool = Arc::clone(&pool);
                if let Err(_) = pool.execute(move || {
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
        let pool_size = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(250);
        run_pooled_server(pool_size);
    } else {
        run_spawning_server();
    }
}
