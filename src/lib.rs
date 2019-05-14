use std::env;
use std::io;
use std::thread;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use chrono::prelude::*;

pub enum Message {
    NewJob(Job),
    Terminate,
}

pub struct Request {
    http_version: String,
    method: String,
    path: String,
    time: DateTime<Local>,
}

/// A ThreadPool is a vector of workers and a sender.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

// Let's type-alias Job so it runs in a box.
type Job = Box<FnBox + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The 'new' function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    /// Execute closure.
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Terminate threads in the ThreadPool when it becomes out of scope.
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a Worker object.
    ///
    /// **id** is the unique ID for the worker, **receiver** is the channel for its jobs.
    ///
    /// # Panics
    ///
    /// The new() function will panic if the receiver was poisoned.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>)
        -> Worker {

        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);

                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);

                        break;
                    },
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// Decide how to respond, and what HTML to render, to the contents of the TcpStream.
pub fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let mut request_line = String::new();
    let mut message = String::new();

    for line in buffer.lines() {
        request_line = line.unwrap();
        break;
    }

    match parse_request(&mut request_line) {
        Ok(request) => {
            log_request(&request);
            let current_user = env::var("USER").unwrap();

            let file = if request.path == "/" {
                File::open(format!("/home/{}/.config/tinyserve/index.html", current_user))
            } else if request.path == "/pi" {
                File::open(format!("/home/{}/.config/tinyserve/pi.html", current_user))
            }  else {
                File::open(format!("/home/{}/.config/tinyserve{}", current_user, request.path))
            };
            let mut status = String::new();
            let mut file = match file {
                Ok(file) => {
                    status = "HTTP/1.1 200 OK\r\n".to_string();
                    file
                },
                Err(_error) => {
                    status = "HTTP/1.1 404 NOT FOUND\r\n".to_string();
                    File::open(format!("/home/{}/.config/tinyserve/404.html", current_user)).unwrap()
                }
            };
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            // let tokens = request.path.split(".");
            let extension = if status.starts_with("HTTP/1.1 404") || request.path == "/" || request.path == "/pi" {
                "html"
            } else {
                request.path.split(".").last().unwrap()
            };
            let content_type = if extension == "js" {
                String::from("javascript")
            } else {
                String::from(extension)
            };

            message = format!("{}Content-Type: text/{}\r\n\r\n{}", status, content_type, contents);
        },
        Err(()) => {
            eprintln!("Bad request! {}", &request_line);
        }
    }
    stream.write(message.as_bytes()).unwrap();
    stream.flush().unwrap();

    Ok(())
}

fn parse_request(request: &mut String) -> Result<Request, ()> {
    let mut parts = request.split(" ");
    let method = match parts.next() {
        Some(method) => method.trim().to_string(),
        None => return Err(()),
    };
    let path = match parts.next() {
        Some(path) => path.trim().to_string(),
        None => return Err(()),
    };
    let http_version = match parts.next() {
        Some(version) => version.trim().to_string(),
        None => return Err(()),
    };
    let time = Local::now();

    Ok( Request {
        http_version: http_version,
        method: method,
        path: path,
        time: time
    } )
}

fn log_request(request: &Request) {
    println!(
        "[{}] \"{} {} {}\"",
        request.time,
        request.method,
        request.path,
        request.http_version,
    );
}

