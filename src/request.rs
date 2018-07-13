use std::{
	io,io::Read,io::Write,
	net::{SocketAddr,TcpStream},
	str
};
use super::os_windows;

pub struct Request {
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

impl Request {
	pub(crate) fn new(mut stream : TcpStream, addr : SocketAddr) -> Result<Request,::std::io::Error> {
		// Read all data now, since we only expect simple requests like "HTTP 1.0 GET /"
		let data = try!(read_all(&mut stream));

		// Extract the request
		let request = extract_request_url(&data);

		Ok(Request {
			stream: stream,
			addr: addr,
			request: match request {
				Some(s) => s.into(),
				None => None
			}
		})
	}

	/// Return the address of the requesting client.
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

	pub fn respond_ok(&mut self, data: &[u8]) -> io::Result<usize> {
		self.respond("200 OK", data, None)
	}

	pub fn respond(&mut self, status_code: &str, data: &[u8], headers: Option<&Vec<String>>) -> io::Result<usize> {
		// Write status line
		let mut bytes_written =
			try!(self.stream.write(format!("HTTP/1.0 {}\r\nContent-Length: {}\r\n\r\n", status_code, data.len()).as_bytes()));
		if headers.is_some() {
			for h in headers.unwrap() {
				bytes_written += try!(self.stream.write(format!("{}\r\n", h).as_bytes()));
			}
		}

		Ok(bytes_written)
	}
}
