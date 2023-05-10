use hello::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn main() {
    // Bind func works line the new func in that it returns a new TcpListener inst.
    // bind func returns a Result<T, E>
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4); //creates new thread pool with configurable number of threads.
    
    // incoming returns an iterator that gives us a sequence of streams
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        
        // Takes a closure the pool should run.
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    
    println!("Shutting down.");
}

// Handle function will implement functionality to read a request from the browser
// and return a HTML response as a body
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap(); // read contents of a file to a string.

    // Response variable that holds the success message's data as the body
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    // Call as_bytes() on response to convert the string data to bytes.
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
