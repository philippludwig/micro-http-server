# The µHTTP server
The µHTTP server is a very small HTTP server implementation for Rust
without the needed for complicated stuff like Futures.

## Motivation
For various small projects, I usually need a tiny HTTP server which
just answers a few very small API-like requests. Since the web
frameworks that are available for Rust all are heavily undocumented
beside some small "getting started" examples, I turned to supposedly
low-level frameworks like hyper, which also is severly lacking in
documentation and examples; therefore I created µHTTP which does
everything I need.

## Changelog

* 0.0.3: Added support for OS X, thanks to [SteamPoweredAnimal](https://github.com/SteamPoweredAnimal).

## Features
* HTTP/1.0 GET requests
* Custom headers & responses

That's it. If you need more, feel free to open an Issue or a PR.

## Usage
Creating a server is as simple as this:

```
let server = MicroHTTP::new("127.0.0.1:3000")
	.expect("Could not create server.");
```

Now you can check if a client has connected:

```
let client = server.next_client().unwrap();
if client.is_some() {
	// New connection
} else {
	// No one talked to the server :(
}
```

To retrieve the client's request, call ``request()``:

```
let request_str = client.request()
	.expect("Client didn't request anything!);
```

To respond, use ``respond_ok`` for _HTTP 200_ or
``respond`` for a more custom response:

```
client.respond_ok("I got your request".as_bytes());
...
client.respond("200 OK",
	"Here is my custom response".as_bytes(),
	vec!(
		"Some-Header: Some Value",
		"Some other Header: Some other Value"
	));
```

## Example + documentation
* Inside the repository, in the folder _examples_ you can currently
find one example for a simple echo server.
* This crate is compiled with ``#![deny(missing_docs)]``, since everything should be documented - otherwise it is useless.
* If you have any problems with the documentation, please open an issue.
