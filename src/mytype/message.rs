
// Define a Application-level Message struct to represent a network packet.
#[derive(Debug, Eq)]
pub struct Message {
    pub seq_num: u64,
    pub source_line: u8,
    pub ts: u64,
    data: Vec<u8>
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

// To be stored in a BTreeSet, Message needs to be comparable.
// We will compare messages based solely on their sequence number, which also
// defines their uniqueness within the set.
impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.seq_num.cmp(&other.seq_num)
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.seq_num == other.seq_num
    }
}
