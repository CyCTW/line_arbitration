// Since this is an integration test in the `tests` directory, it's treated
// as an external user of the library. We need to import the public items
// from our `line_arbitration` crate.
use line_arbitration::arbiter::{Arbiter, ArbiterError};
use line_arbitration::mytype::message::Message;

// Helper function to create messages for tests
fn msg(seq_num: u64, source_line: u8) -> Message {
    Message::new(seq_num, source_line, 0, vec![])
}

#[test]
fn test_in_order_messages() {
    let mut arbiter = Arbiter::new(2, 5);
    let messages = arbiter.receive_message(msg(1, 0)).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].seq_num, 1);

    let messages = arbiter.receive_message(msg(2, 1)).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].seq_num, 2);
}

#[test]
fn test_out_of_order_buffering_and_gap_filling() {
    let mut arbiter = Arbiter::new(2, 5);
    // Message 3 arrives, but 1 and 2 are missing. Should be buffered.
    let messages = arbiter.receive_message(msg(3, 0)).unwrap();
    assert!(messages.is_empty(), "Should buffer message 3 and return nothing");

    // Message 2 arrives, also buffered.
    let messages = arbiter.receive_message(msg(2, 1)).unwrap();
    assert!(messages.is_empty(), "Should buffer message 2 and return nothing");

    // Send 1, filling the gap.
    let messages = arbiter.receive_message(msg(1, 0)).unwrap();
    // Should receive 1, 2, and 3 in order.
    assert_eq!(messages.len(), 3, "Should return the complete sequence");
    assert_eq!(messages[0].seq_num, 1);
    assert_eq!(messages[1].seq_num, 2);
    assert_eq!(messages[2].seq_num, 3);
}

#[test]
fn test_stale_and_duplicate_messages() {
    let mut arbiter = Arbiter::new(2, 5);
    arbiter.receive_message(msg(1, 0)).unwrap();
    arbiter.receive_message(msg(2, 1)).unwrap();

    // Stale message
    let messages = arbiter.receive_message(msg(1, 0)).unwrap();
    assert!(messages.is_empty(), "Should discard stale message 1");

    // Duplicate message (already processed)
    let messages = arbiter.receive_message(msg(2, 1)).unwrap();
    assert!(messages.is_empty(), "Should discard duplicate message 2");

    // Stale message (seq_num 0)
    let messages = arbiter.receive_message(msg(0, 0)).unwrap();
    assert!(messages.is_empty(), "Should discard stale message 0");
}

#[test]
fn test_out_of_bounds_source_line() {
    let mut arbiter = Arbiter::new(2, 5);
    let result = arbiter.receive_message(msg(1, 2)); // source_line 2 is out of bounds for num_lines=2
    assert_eq!(result.unwrap_err(), ArbiterError::OutOfBoundsSourceLine);
}

#[test]
fn test_unrecoverable_gap() {
    let mut arbiter = Arbiter::new(2, 3); // num_lines=2, threshold=3

    // Create a gap at seq_num 1 by receiving messages with higher seq_nums
    arbiter.receive_message(msg(2, 0)).unwrap();
    arbiter.receive_message(msg(3, 1)).unwrap();

    // Advance both lines past the unrecoverable threshold for gap 1.
    // The gap is at seq_num 1. Threshold is 3. Need all lines >= 1 + 3 = 4.
    arbiter.receive_message(msg(4, 0)).unwrap();
    let result = arbiter.receive_message(msg(5, 1));

    assert_eq!(result.unwrap_err(), ArbiterError::UnrecoverableGap);
}

#[test]
fn test_multiple_gaps() {
    let mut arbiter = Arbiter::new(1, 10);
    // Create gaps for 1, 3, 4 by sending 2, 5, and 6
    assert!(arbiter.receive_message(msg(2, 0)).unwrap().is_empty());
    assert!(arbiter.receive_message(msg(5, 0)).unwrap().is_empty());
    assert!(arbiter.receive_message(msg(6, 0)).unwrap().is_empty());

    // Fill gap for 1, which should release 1 and 2
    let messages = arbiter.receive_message(msg(1, 0)).unwrap();
    assert_eq!(messages.len(), 2, "Should return 1 and 2");
    assert_eq!(messages[0].seq_num, 1);
    assert_eq!(messages[1].seq_num, 2);

    // Fill gap for 3, which should release only 3
    let messages = arbiter.receive_message(msg(3, 0)).unwrap();
    assert_eq!(messages.len(), 1, "Should return 3");
    assert_eq!(messages[0].seq_num, 3);

    // Fill gap for 4, which should release 3, 4, 5, and 6
    let messages = arbiter.receive_message(msg(4, 0)).unwrap();
    assert_eq!(messages.len(), 3, "Should return 4, 5, 6");
    assert_eq!(messages[0].seq_num, 4);
    assert_eq!(messages[1].seq_num, 5);
    assert_eq!(messages[2].seq_num, 6);
}

#[test]
fn test_duplicate_buffered_message() {
    let mut arbiter = Arbiter::new(1, 10);
    // Buffer message 3
    assert!(arbiter.receive_message(msg(3, 0)).unwrap().is_empty());

    // Try to buffer a duplicate of message 3
    assert!(arbiter.receive_message(msg(3, 0)).unwrap().is_empty());

    // The BTreeSet in the buffer should have ignored the duplicate.
    // We can't check the buffer size directly, but we can see the output when we fill the gap.
    let messages = arbiter.receive_message(msg(1, 0)).unwrap();
    assert_eq!(messages.len(), 1); // Still get 1
    assert_eq!(messages[0].seq_num, 1);


    let messages = arbiter.receive_message(msg(2, 0)).unwrap();
    assert_eq!(messages.len(), 2); // Should get 2 and 3, but not two 3s.
    assert_eq!(messages[0].seq_num, 2);
    assert_eq!(messages[1].seq_num, 3);
}

#[test]
fn test_unrecoverable_gap_threshold_not_met() {
    let mut arbiter = Arbiter::new(2, 5); // threshold = 5

    // Create a gap at seq_num 1
    arbiter.receive_message(msg(2, 0)).unwrap();

    // Advance lines, but not enough to trigger the error.
    // Gap is at 1, threshold is 5. Error triggers when all lines >= 1 + 5 = 6.
    arbiter.receive_message(msg(5, 0)).unwrap(); // Line 0 is at 5
    let messages = arbiter.receive_message(msg(4, 1)).unwrap(); // Line 1 is at 4

    // No error should occur, and no messages should be released.
    assert!(messages.is_empty());
}

#[test]
fn test_recovery_from_unrecoverable_gap() {
    let mut arbiter = Arbiter::new(2, 3); // threshold = 3

    // Create a gap at seq_num 1
    arbiter.receive_message(msg(2, 0)).unwrap();
    arbiter.receive_message(msg(5, 1)).unwrap(); // Buffer msg 5

    // Trigger the unrecoverable gap error.
    // Gap is at 1, threshold is 3. Error triggers when all lines >= 1 + 3 = 4.
    let result = arbiter.receive_message(msg(4, 0));
    assert_eq!(result.unwrap_err(), ArbiterError::UnrecoverableGap);

}