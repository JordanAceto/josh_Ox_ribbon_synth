use crate::board::Board;
use crate::midi_message::MidiMessage;
use heapless::Vec;

const MESSAGE_QUEUE_MAX_LEN: usize = 3;

/// A very basic MIDI transmitter is represented here.
pub struct MidiTransmitter {
    queue: Vec<MidiMessage, MESSAGE_QUEUE_MAX_LEN>,
}

impl MidiTransmitter {
    /// `MidiTransmitter::new()` is a new midi transmitter
    ///
    /// # Arguments:
    ///
    /// * `channel` - The MIDI channel to use
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    /// `mt.push(m)` pushes the MIDI message `m` onto the message queue
    pub fn push(&mut self, msg: MidiMessage) {
        self.queue.push(msg).ok();
    }

    /// `mt.send_queue(b)` sends all MIDI messages currently in the queue via the board serial port
    pub fn send_queue(&mut self, board: &mut Board) {
        // all MIDI messages so far have length of 3, this might change if we add more complex MIDI behavior
        let vec_of_bytes: Vec<u8, { MESSAGE_QUEUE_MAX_LEN * 3 }> = self
            .queue
            .iter()
            .map(|msg| msg.as_bytes())
            .flatten()
            .collect();
        board.serial_write_all(&vec_of_bytes[..]);
        self.queue.clear();
    }
}
