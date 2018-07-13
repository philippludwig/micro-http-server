#![deny(missing_docs)]

//! # Micro HTTP Server
//!
//! The micro HTTP server - may also be called µHTTP - is a small asynchronous HTTP server
//! implementation without Futures or
//! any other overly complicated stuff; therefore, it is ideal for quick prototyping
//! or API-like requests (e.g. exchaning JSON data).
//!
//! µHTTP does not support any kind of load balancing or threading - you
//! would have to implement this yourself if you want it.
//!
//! At the moment, µHTTP only supports GET requests; if you need PUT/POST/ etc.,
//! feel free to create an issue or a pull request!
//!
//! # Example
//!
//! ```
//! use std::{io::{Read,Write},net::TcpStream};
//! use micro_http_server::MicroHTTP;
//!
//! // Create a server on 127.0.0.1:3000.
//! let server = MicroHTTP::new("127.0.0.1:3000").expect("Could not create server.");
//! println!("[Server] Waiting for a client @ 127.0.0.1:3000...");
//!
//! // Client side: Connect to it and request a file.
//! let mut connection = TcpStream::connect("127.0.0.1:3000").expect("Could not reach server");
//! println!("[Client] Connected! - Requesting /cat.txt...");
//! connection.write("GET /cat.txt\r\n\r\n".as_bytes());
//!
//! {
//! 	// Server side: Get request and send a response.
//!     let mut client = server.next_request().unwrap().unwrap();
//!     println!("[Server] Client requested: {}", client.request().as_ref().unwrap());
//!     let bytes_written = client.respond_ok("Cats are nice.\n".as_bytes()).unwrap();
//!     println!("[Server] Sent {} bytes to the client.", bytes_written);
//! } // client is dropped here to close the TcpStream.
//!
//! // Read response
//! let mut buf = String::new();
//! connection.read_to_string(&mut buf);
//! println!("[Client] Content of cat.txt: {}", buf);
//! ```

#[macro_use] extern crate log;

mod microhttp;
mod request;

pub use microhttp::MicroHTTP;
pub use request::Request;

#[cfg(target_os="linux")]
fn os_windows() -> bool { false }

#[cfg(target_os="windows")]
fn os_windows() -> bool { true }



