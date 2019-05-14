extern crate tinyserve;
extern crate chrono;
use tinyserve::*;
use std::env;
use std::io;
use std::net::TcpListener;

// Listen for requests on localhost, port 8000, and create a new job for each incoming stream.
fn main() -> io::Result<()> {
    let _args: Vec<_> = env::args().collect();

    let listener = TcpListener::bind("127.0.0.1:8000");
    let listener = match listener {
        Ok(listener) => {
            let pool = ThreadPool::new(8);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        pool.execute(|| {
                            handle_client(stream).unwrap();
                        })
                    }
                    Err(_e) => {
                        panic!("Connection failed");
                    }
                }
            }
        }
        Err(_e) => {
            panic!("Address 127.0.0.1:8000 in use, is Tinyserve already running?");
        }
    };

    println!("Shutting down.");
    Ok(())
}

