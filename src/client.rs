use std::{
	io,io::Read,io::Write,
	net::{SocketAddr,TcpStream},
	str
};
use super::os_windows;

/// This struct represents a client which has connected to the µHTTP server.microhttp
///
/// If an instance of this struct is dropped, the connection is closed.
#[derive(Debug)]
pub struct Client {
	stream: TcpStream,
	addr: SocketAddr,
	request: Option<String>
}

// Read all data from an incoming stream
fn read_all(stream: &mut TcpStream) -> Result<Vec<u8>,io::ErrorKind> {
	let mut result = Vec::new();

	loop {
		const BUF_SIZE: usize = 4096;
		let mut buf: [u8; BUF_SIZE] = [0u8; BUF_SIZE];
		match stream.read(&mut buf) {
			Ok(val) => if val > 0 {
				result.append(&mut Vec::from(&buf[0..val]));
				if val < BUF_SIZE {
					return Ok(result);
				}
			} else {
				// Stop reading if we don't have anything left to
				// read at the moment.
				return Ok(result);
			},
			Err(e) => match e.kind() {
				::std::io::ErrorKind::WouldBlock => return Ok(result),
				::std::io::ErrorKind::TimedOut => match os_windows() {
					true => return Ok(result),
					false => return Err(::std::io::ErrorKind::TimedOut)
				},
				kind => return Err(kind)
			}
		};
	}
}

fn extract_request_url(buf: &[u8]) -> Option<String> {
	let s = str::from_utf8(buf).unwrap();

	for line in s.split("\r\n") {
		if line.starts_with("GET ") {
			let components = line.split(" ").collect::<Vec<&str>>();
			if components.len() < 2 {
				warn!("Invalid GET line: {}", line);
				continue;
			}
			return Some(String::from(*components.get(1).unwrap()));
		}
	}

	None
}

impl Client {
	pub(crate) fn new(mut stream : TcpStream, addr : SocketAddr) -> Result<Client,::std::io::Error> {
		// Read all data now, since we only expect simple requests like "HTTP 1.0 GET /"
		let data = read_all(&mut stream)?;

		// Extract the request
		let request = extract_request_url(&data);

		Ok(Client {
			stream: stream,
			addr: addr,
			request: match request {
				Some(s) => s.into(),
				None => None
			}
		})
	}

	/// Return the address of the requesting client, for example "1.2.3.4:9435".
	pub fn addr(&self) -> SocketAddr {
		self.addr
	}

	/// Return the request the client made or None if the client
	/// didn't make any or an invalid one.
	///
	/// **Note**: At the moment, only HTTP GET is supported.
	/// Any other requests will not be collected.
	pub fn request(&self) -> &Option<String> {
		&self.request
	}

	/// Send a HTTP 200 OK response to the client + the provided data.
	/// The data may be an empty array, for example the following
	/// implementation echos all requests except "/hello":
	///
	/// Consider using ``respond_ok_chunked`` for sending file-backed data.
	///
	/// ```
	/// use micro_http_server::MicroHTTP;
	/// use std::io::*;
	/// let server = MicroHTTP::new("127.0.0.1:4000").expect("Could not create server.");
	/// # let mut connection = ::std::net::TcpStream::connect("127.0.0.1:4000").unwrap();
	/// # connection.write("GET /\r\n\r\n".as_bytes());
	/// let mut client = server.next_client().unwrap().unwrap();
	/// let request_str: String = client.request().as_ref().unwrap().clone();
	///
	/// match request_str.as_ref() {
	/// 	"/hello" => client.respond_ok(&[]),
	///     _ => client.respond_ok(request_str.as_bytes())  // Echo request
	/// };
	/// ```
	pub fn respond_ok(&mut self, data: &[u8]) -> io::Result<usize> {
		self.respond_ok_chunked(data, data.len())
	}

	// The test in this doc comment is no_run because it refers to an arbitrary
	// file that may not exist on the current system.

	/// Send a HTTP 200 OK response to the client + the provided data.
	/// The data may be any type implementing [Read](std::io::Read) and
	/// will be read in chunks. This is useful for serving file-backed
	/// data that should not be loaded into memory all at once.
	///
	/// ```no_run
	/// use micro_http_server::MicroHTTP;
	/// use std::io::*;
	/// use std::fs::*;
	/// let server = MicroHTTP::new("127.0.0.1:4000").expect("Could not create server.");
	/// # let mut connection = ::std::net::TcpStream::connect("127.0.0.1:4000").unwrap();
	/// # connection.write("GET /\r\n\r\n".as_bytes());
	/// let mut client = server.next_client().unwrap().unwrap();
	/// client.request();
	///
	/// let mut file_handle = OpenOptions::new()
	///		.read(true)
	///		.write(false)
	///		.open("/some/local/file")
	///		.unwrap();
	///	let file_len = file_handle.metadata().unwrap().len() as usize;
	///
	/// client.respond_ok_chunked(file_handle, file_len);
	///
	/// ```
	pub fn respond_ok_chunked(&mut self, data: impl Read, content_size: usize) -> io::Result<usize> {
		self.respond_chunked("200 OK", data, content_size, &vec!())
	}

	/// Send response data to the client.
	///
	/// This is similar to ``respond_ok``, but you may control the details yourself.
	///
	/// Consider using ``respond_chunked`` for sending file-backed data.
	///
	/// # Parameters
	/// * ``status_code``: Select the status code of the response, e.g. ``200 OK``.
	/// * ``data``: Data to transmit. May be empty.
	/// * ``headers``: Additional headers to add to the response. May be empty.
	///
	/// Calling ``respond("200 OK", data, &vec!())`` is the same as calling ``respond_ok(data)``.
	pub fn respond(
		&mut self,
		status_code: &str,
		data: &[u8],
		headers: &Vec<String>) -> io::Result<usize>
	{
		self.respond_chunked(status_code, data, data.len(), headers)
	}

	/// Send repsonse data to the client.
	/// 
	/// This is similar to ``respond_ok_chunked``, but you may control the details
	/// yourself.
	///
	/// # Parameters
	/// * ``status_code``: Select the status code of the response, e.ge ``200 OK``.
	/// * ``data``: Data to transmit. May be empty
	/// * ``content_size``: Size of the data to transmit in bytes
	/// * ``headers``: Additional headers to add to the response. May be empty.
	///
	/// Calling ``respond_chunked("200 OK", data, content_size, &vec!())`` is the same as calling
	/// ``repsond_ok_chunked(data, content_size)``.
	pub fn respond_chunked(
		&mut self,
		status_code: &str,
		mut data: impl Read,
		content_size: usize,
		headers: &Vec<String>) -> io::Result<usize> 
	{
		// Write status line
		let mut bytes_written =
			self.stream.write(format!("HTTP/1.0 {}\r\nContent-Length: {}\r\n", status_code, content_size).as_bytes())?;

		for h in headers {
			bytes_written += self.stream.write(format!("{}\r\n", h).as_ref())?;
		}
		bytes_written += self.stream.write("\r\n".as_bytes())?;

		let mut buffer = [0; Self::CHUNK_SIZE];
		loop {
			let bytes_read = data.read(&mut buffer)?;
			if bytes_read == 0 { break; }
			bytes_written += self.stream.write(&buffer[..bytes_read])?;
		}

		Ok(bytes_written)
	}

	const CHUNK_SIZE: usize = 4096;
}
