extern crate sse_client;
use sse_client::EventSource;

fn main() {
    let event_source = EventSource::new("http://localhost:3000/sse").unwrap();

    for event in event_source.receiver().iter() {
        println!("New Message: {}", event.data);
    }
}
