extern crate micro_http_server;
use micro_http_server::MicroHTTP;

pub fn main() {
	let server = MicroHTTP::new("127.0.0.1:3000")
		.expect("Could not create server.");

	println!("Waiting for requests on: http://127.0.0.1:3000");

	loop {
		let result = server.next_client();
		if result.is_err() {
			println!("Server failed: {:?}", result);
			break;
		}

		match result.unwrap() {
			None => ::std::thread::sleep(::std::time::Duration::from_millis(500)),
			Some(mut client) => {
				if client.request().is_none() {
					println!("Client {} didn't send any request", client.addr());
					client.respond_ok("No request :(".as_bytes())
						.expect("Could not send data to client!");
				} else {
					let request_copy = client.request().as_ref().unwrap().clone();

					println!("Client {} requested {}, echoing...", client.addr(), request_copy);
					client.respond_ok(request_copy.as_bytes())
						.expect("Could not send data to client!");
				}
			}
		}
	}
}
