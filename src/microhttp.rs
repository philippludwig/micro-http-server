use std::{io, net::TcpListener};

use request::Request;

/// STUB: MicroHTTP struct doc
pub struct MicroHTTP {
	// Internal listener which is used for the server part
	listener: TcpListener,
}

impl MicroHTTP {
	/// Create a new MicroHTTP server on the given interface.microhttp
	///
	/// Internally, this just tries to create a ``TcpListener`` - nothing special.
	/// Returns the new MicroHTTP server or an ``std::io::Error`` on error.
	///
	/// # Example
	///
	/// ```
	/// use micro_http_server::MicroHTTP;
	///
	/// let interface: &str = "127.0.0.1:3000";
	/// let server = MicroHTTP::new(interface)
	///     .expect("Could not create server, maybe the port is already being used?");
	/// ```
	pub fn new(interface: &str) -> Result<MicroHTTP,io::Error> {
		// Create listener using the requested interface
		let listener = try!(TcpListener::bind(interface));

		// Set to non-blocking so we can later check if we have requests
		// without blocking the whole thread.
		try!(listener.set_nonblocking(true));

		// Return created instance
		Ok(MicroHTTP {
			listener : listener
		})
	}

	/// Return the next available request which is incoming at this server.
	///
	/// Returns either:
	/// * Some(Request) if a request is available
	/// * None if no request is currently available (i.e. no client has reached out to the server yet)
	/// * std::io::Error if something is wrong with the server.
	///
	/// # Example
	///
	/// ```
	/// use std::{net::TcpStream,thread};
	/// use micro_http_server::MicroHTTP;
	///
	/// let server = MicroHTTP::new("127.0.0.1:3000").expect("Could not create server.");
	/// thread::spawn(move || {
	///     println!("Waiting for a client @ 127.0.0.1:3000...");
	///     loop {
	///         let client = server.next_request().unwrap();
	///         if client.is_some() {
	///             println!("Got a client!");
	///             break;
	///         }
	/// 	}
	///
	///     let connection = TcpStream::connect("127.0.0.1:3000").expect("Could not reach server");
	/// });
	///
	pub fn next_request(&self) -> Result<Option<Request>,io::Error> {
		// See if we have any incoming connections.
		match self.listener.accept() {
			// We do - try to create a Request from the incoming socket & addr,
			// then return it.
			Ok( (socket, addr) ) => Ok(Some(try!(Request::new(socket, addr)))),

			// Check if we just don't have an incoming connection or
			// if really an error occured.
			Err(err) => match err.kind() {
				io::ErrorKind::WouldBlock => Ok(None), // No incoming connection
				_ => Err(err) // We encountered an error :(
			}
		}
	}
}
