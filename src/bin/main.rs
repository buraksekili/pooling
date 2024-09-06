use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Instant, Duration};
use std::thread;

fn main() {
    let num_requests = 10000;
    let num_threads = 100;

    let start = Instant::now();

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..num_requests / num_threads {
                    let mut stream = TcpStream::connect("localhost:7878").unwrap();
                    stream.write(b"Hello").unwrap();
                    let mut buffer = [0; 512];
                    stream.read(&mut buffer).unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("Total time: {:?}", duration);
    println!("Requests per second: {}", num_requests as f64 / duration.as_secs_f64());
}
