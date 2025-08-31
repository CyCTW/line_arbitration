use crate::arbiter::Arbitratable;

// Define a Application-level Message struct to represent a network packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub seq_num: u64,
    pub source_line: u8,
    pub ts: u64,
    /// The payload of the message.
    pub data: Vec<u8>,
}

// An implementation block is where you define functions (methods and
// associated functions) for your struct.
impl Message {
    /// Creates a new Message.
    /// This is an "associated function" because it's associated with the `Message`
    /// type. It's common to use `new` as a constructor.
    pub fn new(seq_num: u64, source_line: u8, ts: u64, data: Vec<u8>) -> Self {
        // `Self` is an alias for the type this `impl` block is for (i.e., `Message`).
        Message {
            seq_num, // Using field init shorthand because param name matches field name
            source_line,
            ts,
            data,
        }
    }
}

impl Arbitratable for Message {
    fn seq_num(&self) -> u64 {
        self.seq_num
    }

    fn source_line(&self) -> u8 {
        self.source_line
    }
}
