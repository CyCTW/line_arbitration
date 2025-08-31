# Line Arbitration

A Rust library for arbitrating and re-ordering sequential messages from multiple sources into a single, ordered stream.

This library is designed for scenarios where you receive multiple streams of data that are individually ordered (e.g., by a sequence number) but may arrive out-of-order relative to each other. The `Arbiter` processes these messages, buffers them when necessary, and emits a single, strictly sequential stream.

## Core Concepts

Imagine you have multiple independent data feeds (or "lines"), each producing messages with a unique, incrementing sequence number. Due to network latency or other transport-level issues, these messages can arrive at a central processor out of order.

The `Arbiter`'s job is to:
1.  Receive messages from any line.
2.  If a message is the next one expected in the global sequence, emit it immediately.
3.  If a message arrives "from the future" (i.e., there's a gap in the sequence), buffer it.
4.  When a message arrives that fills a gap, emit it and any subsequent contiguous messages from the buffer.
5.  Detect and report when a gap is "unrecoverable," meaning it's highly likely the missing message will never arrive.

## Features

- **Sequential Re-ordering:** Correctly orders messages from multiple sources based on a `seq_num`.
- **Out-of-Order Buffering:** Temporarily stores messages that arrive before their predecessors.
- **Gap Filling:** Automatically releases buffered messages once a missing message in the sequence arrives.
- **Unrecoverable Gap Detection:** Identifies situations where a message is likely lost forever and signals an `ArbiterError::UnrecoverableGap`. This is based on a configurable threshold where all source lines have produced messages far beyond the gap.
- **Duplicate & Stale Message Handling:** Efficiently ignores messages that have already been processed or are older than the current sequence.
- **Type-Safe Error Handling:** Uses a dedicated `ArbiterError` enum for clear and robust error management.

## Usage

First, add `line_arbitration` to your `Cargo.toml` dependencies.

Here is an example demonstrating how the `Arbiter` handles out-of-order messages and fills a gap.

```rust
use line_arbitration::arbiter::{Arbiter, ArbiterError};
use line_arbitration::mytype::message::Message;

fn main() -> Result<(), ArbiterError> {
    // Initialize an Arbiter to handle 2 source lines.
    // A gap is considered unrecoverable if all lines have produced messages
    // 5 sequence numbers past the gap.
    let mut arbiter = Arbiter::new(2, 5);

    // Helper to create messages easily
    let msg = |seq_num, source_line| Message::new(seq_num, source_line, 0, vec![]);

    // --- Scenario: Out-of-order delivery ---

    // 1. Message 3 arrives first. A gap exists for 1 and 2.
    //    It gets buffered, and nothing is returned yet.
    let messages = arbiter.receive_message(msg(3, 0))?;
    assert!(messages.is_empty());
    println!("Received message 3, buffered. No output yet.");

    // 2. Message 2 arrives next. It also gets buffered.
    let messages = arbiter.receive_message(msg(2, 1))?;
    assert!(messages.is_empty());
    println!("Received message 2, buffered. No output yet.");

    // 3. Message 1 arrives, filling the initial gap.
    //    The arbiter can now release the sequence 1, 2, 3.
    let messages = arbiter.receive_message(msg(1, 0))?;
    println!("Received message 1, filling the gap!");

    // The returned vector contains the complete, ordered sequence.
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].seq_num, 1);
    assert_eq!(messages[1].seq_num, 2);
    assert_eq!(messages[2].seq_num, 3);

    println!("Output stream: {:?}", messages.iter().map(|m| m.seq_num).collect::<Vec<_>>());

    Ok(())
}
```

## API

### `Arbiter<T: Arbitratable>`
The main struct that performs the arbitration. It is generic over any type `T` that implements the `Arbitratable` trait.

### `Arbiter::new(num_lines: usize, unrecoverable_threshold: u64) -> Arbiter<T>`
Creates a new `Arbiter`. The type `T` is inferred from usage.
- `num_lines`: The total number of source lines to track.
- `unrecoverable_threshold`: The number of messages past a gap for all lines before the gap is considered unrecoverable.

### `arbiter.receive_message(&mut self, msg: T) -> Result<Vec<T>, ArbiterError>`
Processes an incoming message.
- `msg`: An instance of a type `T` that implements `Arbitratable`.
- Returns `Ok(Vec<T>)` containing zero or more messages that are now in-order and ready for consumption.
- Returns `Err(ArbiterError)` if an issue occurs.

### `trait Arbitratable: Clone`
A trait that allows your custom message types to be used with the `Arbiter`.

```rust
pub trait Arbitratable: Clone {
    fn seq_num(&self) -> u64;
    fn source_line(&self) -> u8;
}
```

**Important**: To function correctly with the `Arbiter`'s internal buffer, your type's implementation of `Ord` and `PartialEq` **must** be based on the sequence number returned by `seq_num()`. The `line_arbitration::mytype::message::Message` struct provides a reference implementation.

### `ArbiterError`
- `OutOfBoundsSourceLine`: The `source_line` on a message was greater than or equal to `num_lines`.
- `UnrecoverableGap`: A gap in the sequence has been deemed unrecoverable based on the configured threshold.

## Building and Testing

To build the library and run the tests, use the standard Cargo commands.

```bash
# Build the project
cargo build

# Run tests (including integration tests)
cargo test
```
