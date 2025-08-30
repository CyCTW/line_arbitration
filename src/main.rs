// Import items from our own library crate
use line_arbitration::mytype::message::Message;
use line_arbitration::arbiter::Arbiter;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_micros() as u64;

    let msg = Message::new(0, 1, current_time, "Hello, world!".as_bytes().to_vec());
    println!("{:?}", msg);

    // Initialize the Arbiter
    let arbiter = Arbiter::new(3, 5);
    println!("Initial arbiter state: {:?}", arbiter);
}