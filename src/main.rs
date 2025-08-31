// Import items from our own library crate
use line_arbitration::mytype::message::Message;
use line_arbitration::arbiter::Arbiter;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as u64;

    // Create a sample message with sequence number 1 from source line 0.
    let msg = Message::new(1, 0, current_time, "Hello, world!".as_bytes().to_vec());
    println!("{:?}", msg);

    // Initialize the Arbiter. Because Arbiter is generic over the message type,
    // we must tell the compiler which concrete type it will be handling.
    // let arbiter: Arbiter<Message> = Arbiter::new(3, 5);
    let arbiter: Arbiter<Message> = Arbiter::new(3, 5);
    println!("Initial arbiter state: {:?}", arbiter);
}