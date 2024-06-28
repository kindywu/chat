extern crate sse_client;
use sse_client::EventSource;

fn main() {
    // let url = "http://localhost:3000/sse";
    let url = "http://localhost:3000/sse";
    let event_source = EventSource::new(url).unwrap();

    for event in event_source.receiver().iter() {
        println!("New Message: {}", event.data);
    }
}
