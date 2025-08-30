// We need to bring the Message struct into scope to use it.
use crate::mytype::message::Message;
use std::{collections::BTreeSet};

#[derive(Debug)]
pub struct Arbiter {
    latest_inorder_seq_num: u64, // Track the latest sequence number seen
    latest_seq_nums: Vec<u64>, // Track the latest sequence number seen per line
    buffer: BTreeSet<Message>, // Use a BTreeSet to automatically handle sorting and duplicates.
    unrecoverable_threshold: u64, // If all lines have passed the gap over this threshold, we consider the gap as lost.
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArbiterError {
    OutOfBoundsSourceLine,
    UnrecoverableGap,
}

impl Arbiter {
    /// Creates a new Arbiter.
    ///
    /// * `num_lines`: The total number of source lines to track.
    /// * `unrecoverable_threshold`: The number of messages past a gap for all lines
    ///   before the gap is considered unrecoverable.
    pub fn new(num_lines: usize, unrecoverable_threshold: u64) -> Self {
        Arbiter {
            latest_inorder_seq_num: 0,
            latest_seq_nums: vec![0; num_lines],
            buffer: BTreeSet::new(),
            unrecoverable_threshold,
        }
    }

    pub fn receive_message(&mut self, msg: Message) -> Result<Vec<Message>, ArbiterError> {
        let mut return_messages = vec![];

        // Prevent panic by checking if source_line is valid.
        let line_idx = msg.source_line as usize;
        if line_idx >= self.latest_seq_nums.len() {
            return Err(ArbiterError::OutOfBoundsSourceLine);
        }

        // Correctly update the latest sequence number for the source line.
        self.latest_seq_nums[line_idx] = self.latest_seq_nums[line_idx].max(msg.seq_num);

        if msg.seq_num == self.latest_inorder_seq_num + 1 {
            // Case 1: In-order message.
            self.latest_inorder_seq_num = msg.seq_num;
            return_messages.push(msg);
            // After accepting an in-order message, try to process the buffer.
            return_messages.extend(self.process_buffer());
        } else if msg.seq_num > self.latest_inorder_seq_num + 1 {
            // Case 2: Future message (gap detected).
            // BTreeSet automatically handles sorting and prevents duplicates.
            self.buffer.insert(msg);
        } else {
            // Case 3: Stale or duplicate message. Discard it.
        }

        // Check if any gaps can now be considered unrecoverable.
        self.check_gaps()?;

        Ok(return_messages)
    }

    /// Processes buffered messages that are now in-order.
    fn process_buffer(&mut self) -> Vec<Message> {
        let mut return_messages = vec![];

        while let Some(msg) = self.buffer.first() {
            if msg.seq_num == self.latest_inorder_seq_num + 1 {
                // This message is the one we were waiting for.
                // `BTreeSet::pop_first` is efficient.
                // Safe to unwrap since we just checked with `first()` that it's `Some`.
                let msg = self.buffer.pop_first().unwrap();
                self.latest_inorder_seq_num = msg.seq_num;
                return_messages.push(msg);
            } else {
                // The next message in the buffer is not the one we need, so we stop.
                break;
            }
        }
        
        return_messages
    }

    /// Checks if a gap can be considered unrecoverable.
    pub fn check_gaps(&mut self) -> Result<(), ArbiterError> {
        // If the buffer is empty, there are no gaps to check.
        if self.buffer.is_empty() {
            return Ok(());
        }

        let gap_seq_num = self.latest_inorder_seq_num + 1;

        // Check if all lines have advanced far enough past the current gap.
        let all_lines_passed_gap = self.latest_seq_nums
            .iter()
            .all(|&seq_num| seq_num >= gap_seq_num + self.unrecoverable_threshold);

        if all_lines_passed_gap {
            return Err(ArbiterError::UnrecoverableGap);
        }

        Ok(())
    }
}

