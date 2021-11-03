use std::{io, net::{TcpListener, ToSocketAddrs}};

use client::Client;

/// This is the main struct of the µHTTP server.
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
	pub fn new(interface: impl ToSocketAddrs) -> Result<MicroHTTP,io::Error> {
		// Create listener using the requested interface
		let listener = TcpListener::bind(interface)?;

		// Return created instance
		Ok(MicroHTTP {
			listener : listener
		})
	}

	/// Set whether or not the underlying TcpListener awaits connections in nonblocking mode
	pub fn set_nonblocking(&mut self, state: bool) -> Result<(), io::Error> {
		self.listener.set_nonblocking(state)
	}


	/// Return the next available client which is incoming at this server.
	///
	/// Returns either:
	/// * ``Some(client)`` if a client is available
	/// * ``None`` if no client is currently available (i.e. no one has reached out to the server yet)
	/// * ``std::io::Error`` if something is wrong with the server.
	///
	/// # Example
	///
	/// ```
	/// use std::{io::{Read,Write},net::TcpStream};
	/// use micro_http_server::MicroHTTP;
	///
	/// let server = MicroHTTP::new("127.0.0.1:3000").expect("Could not create server.");
	/// println!("[Server] Waiting for a client @ 127.0.0.1:3000...");
	///
	/// loop {
	///     let result = server.next_client();
	///     if result.is_err() {
	///         println!("Something is wrong with the client: {:?}", result.unwrap_err());
	///         break;
	///     }
	///
	///     match result.unwrap() {
	///         None => {
	///             // Here you can sleep or do something else.
	///             println!("Still waiting for clients...");
	///         },
	///         Some(client) => {
	///             println!("Got a new client from: {:?}", client.addr());
	///         }
	///     }
	/// #    break;
	/// }
	/// ```
	pub fn next_client(&self) -> Result<Option<Client>,io::Error> {
		// See if we have any incoming connections.
		match self.listener.accept() {
			// We do - try to create a Client from the incoming socket & addr,
			// then return it.
			Ok( (socket, addr) ) => Ok(Some(Client::new(socket, addr)?)),

			// Check if we just don't have an incoming connection or
			// if really an error occured.
			Err(err) => match err.kind() {
				io::ErrorKind::WouldBlock => Ok(None), // No incoming connection
				_ => Err(err) // We encountered an error :(
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::MicroHTTP;
	use std::{io::{Read,Write},net::TcpStream};

	#[test]
	fn echo() {
		let server = MicroHTTP::new("127.0.0.1:65534").expect("Could not create server");
		println!("Waiting for a client @ 127.0.0.1:65534...");

		let mut connection = TcpStream::connect("127.0.0.1:65534").expect("Could not reach server");
		println!("Connected!");

		connection.write("GET /\r\n\r\n".as_bytes()).unwrap();

		{
			let opt = server.next_client().unwrap();
			assert_eq!(true, opt.is_some());
			let mut client = opt.unwrap();

			println!("Got a client!");
			assert_eq!(true, client.request().is_some());
			assert_eq!("/", client.request().as_ref().unwrap());
			client.respond_ok("TEST".as_bytes()).unwrap();
		}

		let mut buf = String::new();
		connection.read_to_string(&mut buf).unwrap();
		assert_eq!("HTTP/1.0 200 OK\r\nContent-Length: 4\r\n\r\nTEST", buf);
	}
}
