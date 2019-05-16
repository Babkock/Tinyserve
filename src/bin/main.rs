/*
 * main.rs
 *
 * Tinyserve 0.4
 *
 * Copyright (c) 2019 Tanner Babcock.
 * This software is licensed under the terms of the GNU General Public License. See LICENSE.md for details.
*/
extern crate tinyserve;
extern crate clap;
extern crate chrono;
use tinyserve::*;
use clap::{Arg, App};
use std::io;
use std::net::TcpListener;

// Listen for requests on localhost, port 8000, and create a new job for each incoming stream.
fn main() -> io::Result<()> {
    let matches = App::new("Tinyserve").version("0.4").author("Tanner Babcock <babkock@gmail.com>").about("Tiny multi-threaded web server")
        .arg(Arg::with_name("webroot")
            .short("r")
            .long("webroot")
            .value_name("directory_with_html")
            .help("Sets the web root for the server. index.html is loaded from this directory. Default is ~/.config/tinyserve.")
            .takes_value(true))
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .value_name("127.0.0.1")
            .help("Sets the IPv4 address to bind to. Default is 127.0.0.1 or localhost.")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("8000")
            .help("Sets the port on the specified address. Default is 8000."))
        .arg(Arg::with_name("v")
            .short("v")
            .help("Use -v for verbose output."))
        .get_matches();
   
    let verbose = match matches.occurrences_of("v") {
        0 => false,
        1 => true,
        _ => false
    };
    let address = String::from(matches.value_of("address").unwrap_or("127.0.0.1"));
    let port = String::from(matches.value_of("port").unwrap_or("8000"));

    let listener = TcpListener::bind(format!("{}:{}", address, port));
    match listener {
        Ok(listener) => {
            let pool = ThreadPool::new(8, verbose);

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let matches = matches.clone();
                        let verbose = match matches.occurrences_of("v") {
                            0 => false,
                            1 => true,
                            _ => false
                        };
                        let webroot = String::from(matches.value_of("webroot").unwrap_or("_default_"));

                        pool.execute(move || {
                            handle_client(stream, &webroot, verbose).unwrap();
                        })
                    }
                    Err(_e) => {
                        panic!("Connection failed");
                    }
                }
            }
        }
        Err(_e) => {
            panic!("The specified address {}:{} is in use, is Tinyserve already running?", address, port);
        }
    }

    println!("Shutting down.");
    Ok(())
}

