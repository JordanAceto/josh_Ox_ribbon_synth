use crate::board::Board;

use heapless::Vec;
use midi_convert::{midi_types::MidiMessage, MidiRenderSlice};

const MAX_NUM_MESSAGES_IN_QUEUE: usize = 16;

const MESSAGE_MAX_NUM_BYTES: usize = 3;

const BUFF_LEN: usize = MAX_NUM_MESSAGES_IN_QUEUE * MESSAGE_MAX_NUM_BYTES;

/// A very basic MIDI transmitter is represented here.
pub struct MidiTransmitter {
    queue: Vec<MidiMessage, MAX_NUM_MESSAGES_IN_QUEUE>,
    buffer: [u8; BUFF_LEN],
}

impl MidiTransmitter {
    /// `MidiTransmitter::new()` is a new midi transmitter
    ///
    /// # Arguments:
    ///
    /// * `channel` - The MIDI channel to use
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            buffer: [0; BUFF_LEN],
        }
    }

    /// `mt.push(m)` pushes the MIDI message `m` onto the message queue
    pub fn push(&mut self, msg: MidiMessage) {
        self.queue.push(msg).ok();
    }

    /// `mt.send_queue(b)` sends all MIDI messages currently in the queue via the board serial port
    pub fn send_queue(&mut self, board: &mut Board) {
        let mut i = 0;
        for msg in &self.queue {
            msg.render_slice(&mut self.buffer[i..(i + msg.len())]);
            i += msg.len();
        }
        board.serial_write_all(&self.buffer[..i]);
        self.queue.clear();
    }
}
