use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Instant;
use std::thread;

fn send_request(addr: &str) -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect(addr)?;
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    stream.write(request.as_bytes())?;
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;
    Ok(())
}

fn benchmark(addr: &str, num_requests: usize, num_threads: usize) {
    let requests_per_thread = num_requests / num_threads;
    let start = Instant::now();

    let handles: Vec<_> = (0..num_threads)
        .map(|_| {
            let addr = addr.to_string();
            thread::spawn(move || {
                for _ in 0..requests_per_thread {
                    send_request(&addr).unwrap();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("Total time: {:?}", duration);
    println!("Requests per second: {:.2}", num_requests as f64 / duration.as_secs_f64());
}

fn main() {
    let num_requests = 100;
    let num_threads = 100;

    println!("Benchmarking pooled server:");
    benchmark("localhost:7878", num_requests, num_threads);

    println!("\nBenchmarking spawning server:");
    benchmark("localhost:7879", num_requests, num_threads);
}
