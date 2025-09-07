use std::collections::BTreeMap;

/// A trait for messages that can be processed by the Arbiter.
pub trait Arbitratable: Clone {
    /// Returns the global sequence number of the message.
    fn seq_num(&self) -> u64;
    /// Returns the identifier of the source line this message came from.
    fn source_line(&self) -> u8;
}

#[derive(Debug)]
pub struct Arbiter<T: Arbitratable> {
    latest_inorder_seq_num: u64, // Track the latest sequence number seen
    latest_seq_nums: Vec<u64>, // Track the latest sequence number seen per line
    buffer: BTreeMap<u64, T>, // Use a BTreeMap to automatically handle sorting and prevent duplicates by sequence number.
    unrecoverable_threshold: u64, // If all lines have passed the gap over this threshold, we consider the gap as lost.
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArbiterError {
    OutOfBoundsSourceLine,
    UnrecoverableGap,
}

impl<T: Arbitratable> Arbiter<T> {
    /// Creates a new Arbiter.
    ///
    /// * `num_lines`: The total number of source lines to track.
    /// * `unrecoverable_threshold`: The number of messages past a gap for all lines
    ///   before the gap is considered unrecoverable.
    pub fn new(num_lines: usize, unrecoverable_threshold: u64) -> Self {
        Arbiter {
            latest_inorder_seq_num: 0,
            latest_seq_nums: vec![0; num_lines],
            buffer: BTreeMap::new(),
            unrecoverable_threshold,
        }
    }

    pub fn receive_message(&mut self, msg: T) -> Result<Vec<T>, ArbiterError> {
        let mut return_messages = vec![];

        // Prevent panic by checking if source_line is valid.
        let line_idx = msg.source_line() as usize;
        if line_idx >= self.latest_seq_nums.len() {
            return Err(ArbiterError::OutOfBoundsSourceLine);
        }

        // Correctly update the latest sequence number for the source line.
        self.latest_seq_nums[line_idx] = self.latest_seq_nums[line_idx].max(msg.seq_num());

        if msg.seq_num() == self.latest_inorder_seq_num + 1 {
            // Case 1: In-order message.
            self.latest_inorder_seq_num = msg.seq_num();
            return_messages.push(msg);
            // After accepting an in-order message, try to process the buffer.
            return_messages.extend(self.process_buffer());
        } else if msg.seq_num() > self.latest_inorder_seq_num + 1 {
            // Case 2: Future message (gap detected).
            self.buffer.insert(msg.seq_num(), msg);
        } else {
            // Case 3: Stale or duplicate message. Discard it.
        }

        // Check if any gaps can now be considered unrecoverable.
        self.check_gaps()?;

        Ok(return_messages)
    }

    /// Processes buffered messages that are now in-order.
    fn process_buffer(&mut self) -> Vec<T> {
        let mut return_messages = vec![];

        while let Some(kp) = self.buffer.first_entry() {
            let msg = kp.get();
            if msg.seq_num() == self.latest_inorder_seq_num + 1 {
                // This message is the one we were waiting for.
                // `pop_first` is efficient and safe to unwrap since we just checked that the entry exists.
                let (_, val) = self.buffer.pop_first().unwrap();
                self.latest_inorder_seq_num = val.seq_num();
                return_messages.push(val);
            } else {
                // The next message in the buffer is not the one we need, so we stop.
                break;
            }
        }
        
        return_messages
    }

    /// Checks if a gap can be considered unrecoverable.
    fn check_gaps(&self) -> Result<(), ArbiterError> {
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