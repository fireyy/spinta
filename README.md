# spinta

This is a simple [WebSocket](https://en.wikipedia.org/wiki/WebSocket) library for Rust which can be compiled to both native and web (WASM).

## Usage

``` rust
let receiver = spinta::connect("http://example.com").unwrap();
while let Some(event) = receiver.try_recv() {
    println!("Received {:?}", event);
}
```