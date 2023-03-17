/// A midi message is represented here.
///
/// Not all possible MIDI messages are represented, only those which were useful at the time of writing this module.
pub enum MidiMessage {
    NoteOn(Channel, Note, Velocity),
    NoteOff(Channel, Note, Velocity),
    AllNotesOff(Channel),
    PitchBend(Channel, PitchBendLsb, PitchBendMsb),
}

impl MidiMessage {
    /// `note_on(c, n, v)` is a MIDI note-on message with the specified channel, note number, and velocity
    ///
    /// # Arguments:
    ///
    /// * `ch` - the MIDI channel to use, in `[1..16]`
    ///
    /// * `note` - the MIDI note to use, in `[0..127]`
    ///
    /// * `vel` - the MIDI velocity to use, in `[0..127]`
    pub fn note_on(ch: u8, note: u8, vel: u8) -> MidiMessage {
        MidiMessage::NoteOn(ch.into(), note.into(), vel.into())
    }

    /// `note_off(c, n, v)` is a MIDI note-off message with the specified channel, note number, and velocity
    ///
    /// # Arguments:
    ///
    /// * `ch` - the MIDI channel to use, in `[1..16]`
    ///
    /// * `note` - the MIDI note to use, in `[0..127]`
    ///
    /// * `vel` - the MIDI velocity to use, in `[0..127]`
    pub fn note_off(ch: u8, note: u8, vel: u8) -> MidiMessage {
        MidiMessage::NoteOff(ch.into(), note.into(), vel.into())
    }

    /// `all_notes_off(c)` is a MIDI all-notes-off message with channel `c`
    ///
    /// # Arguments:
    ///
    /// * `ch` - the MIDI channel to use, in `[1..16]`
    pub fn all_notes_off(ch: u8) -> MidiMessage {
        MidiMessage::AllNotesOff(ch.into())
    }

    /// `pitch_bend(c, v)` is the normalized value `v` converted to a MIDI pitch bend message with channel `c`
    ///
    /// # Arguments:
    ///
    /// * `ch` - the MIDI channel to use, in `[1..16]`
    ///
    /// * `val_u14` - The 14 bit pitch bend message to send
    pub fn pitch_bend(ch: u8, val_u14: u16) -> MidiMessage {
        MidiMessage::PitchBend(
            ch.into(),
            (val_u14 as u8).into(),
            ((val_u14 >> 7) as u8).into(),
        )
    }

    /// `mm.as_bytes()` is the MIDI message `mm` converted to a byte array in the correct order to send via serial
    pub fn as_bytes(&self) -> [u8; 3] {
        match self {
            MidiMessage::NoteOn(c, n, v) => [0x90 | c.0, n.0, v.0],
            MidiMessage::NoteOff(c, n, v) => [0x80 | c.0, n.0, v.0],
            MidiMessage::AllNotesOff(c) => [0xB0 | c.0, 0x7B, 0],
            // Explanation for the cheeky OR-1 in the pitch bend LSB: On some MIDI devices if the incoming pitch bend is
            // EXACTLY centered then they are free to update the pitch bend with their own logic. But if it is off by
            // even one, then they use the incoming pitch bend value exactly.
            // If we send exactly centered pitch bend, sometimes the device we are controlling goes out of tune because it
            // "remembers" a trailing pitch bend from before. Always sending a pitch bend message that is ever so slightly off
            // center prevents this. This was only tested using an Arturia Keystep 37 as a MIDI device. Other devices may have
            // different behavior.
            MidiMessage::PitchBend(c, lsb, msb) => [0xE0 | c.0, lsb.0 | 1, msb.0],
        }
    }
}

/// A clamping 7 bit unsigned integer is represented here
///
/// Attempting to create a ClampingU7 with a value larger than `2^7 - 1` will
/// clamp to value `2^7 - 1`
pub struct ClampingU7(u8);

impl From<u8> for ClampingU7 {
    /// `u8_val.into()` is the `u8` as a clamping unsigned 7 bit integer
    fn from(val: u8) -> Self {
        ClampingU7(val.min(U7_MAX))
    }
}

impl Into<u8> for ClampingU7 {
    /// `cu7.into()` is the clamping unsigned 7 bit integer as a `u8`
    fn into(self) -> u8 {
        self.0
    }
}

/// A MIDI channel is represented here, channels range from `[1..16]`
///
/// Attempting to create a Channel with a value larger than `16` will
/// clamp to channel `16`
///
/// Internally the channel is represented as an integer in `[0..15]`
pub struct Channel(u8);

impl From<u8> for Channel {
    fn from(ch: u8) -> Self {
        Self((ch - 1).max(0).min(15))
    }
}

impl Into<u8> for Channel {
    fn into(self) -> u8 {
        self.0
    }
}

use ClampingU7 as Note;
use ClampingU7 as Velocity;
use ClampingU7 as PitchBendMsb;
use ClampingU7 as PitchBendLsb;

/// The maximum value that an unsigned 7 bit integer can take
pub const U7_MAX: u8 = 127;

/// The MIDI maximum velocity value
pub const MAX_VELOCITY: u8 = U7_MAX;

/// The minimum MIDI velocity value
pub const MIN_VELOCITY: u8 = 0;

/// The full scale value of MIDI pitch bend in one direction, pitch bend goes up and down by this amount from the center
pub const PITCH_BEND_FULL_SCALE: u16 = 1 << 13;

/// The center value for pitch bend messages, represents zero pitch bend
pub const PITCH_BEND_CENTER: u16 = PITCH_BEND_FULL_SCALE;
