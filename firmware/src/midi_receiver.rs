use heapless::Vec;

use midi_convert::{
    midi_types::{MidiMessage, Value7},
    MidiByteStreamParser,
};

/// The maximum number of held down MIDI notes we can remember
const HELD_DOWN_NOTE_BUFFER_LEN: usize = 32;

/// A MIDI receiver is represented here.
///
/// The MIDI receiver is fed MIDI data in the form of sequentially bytes and converts this MIDI data into signals used
/// to control music synthesizers or other devices.
pub struct MidiReceiver {
    parser: MidiByteStreamParser,

    // in `[0..127]`
    note_num: u8,

    // in `[0.0, 1.0]`
    velocity: f32,

    // in `[-1.0, 1.0]`
    pitch_bend: f32,

    // in `[0.0, 1.0]`
    mod_wheel: f32,

    // in `[0.0, 1.0]`
    volume: f32,

    // in `[0.0, 1.0]`
    vcf_cutoff: f32,

    // in `[0.0, 1.0]`
    vcf_resonance: f32,

    // in `[0.0, 1.0]`
    portamento_time: f32,

    portamento_enabled: bool,
    sustain_enabled: bool,

    gate: bool,
    rising_gate: bool,
    falling_gate: bool,

    retrigger_mode: RetriggerMode,
    note_priority: NotePriority,

    held_down_notes: Vec<u8, HELD_DOWN_NOTE_BUFFER_LEN>,
}

impl MidiReceiver {
    /// `MidiReceiver::new()` is a new MIDI receiver
    pub fn new() -> Self {
        Self {
            parser: MidiByteStreamParser::new(),

            note_num: 0,

            pitch_bend: 0.0_f32,

            velocity: 0.0_f32,
            mod_wheel: 0.0_f32,
            volume: 0.0_f32,
            vcf_cutoff: 0.0_f32,
            vcf_resonance: 0.0_f32,
            portamento_time: 0.0_f32,

            portamento_enabled: true,
            sustain_enabled: true,

            gate: false,
            rising_gate: false,
            falling_gate: false,

            retrigger_mode: RetriggerMode::NoRetrigger,
            note_priority: NotePriority::Last,

            held_down_notes: Vec::new(),
        }
    }

    /// `mr.parse(b)` parses incoming MIDI data in the form of sequential bytes `b`
    ///
    /// It is expected to call this function every time a new MIDI byte is received.
    pub fn parse(&mut self, byte: u8) {
        match self.parser.parse(byte) {
            Some(MidiMessage::NoteOn(_, note, vel)) => {
                // note-on with velocity of zero is interpreted as note-off
                if 0 == u8::from(vel) {
                    self.handle_note_off(note.into());
                } else {
                    self.handle_note_on(note.into(), vel);
                };
            }
            Some(MidiMessage::NoteOff(_, note, _)) => {
                self.handle_note_off(note.into());
            }
            Some(MidiMessage::PitchBendChange(_, val_u14)) => {
                self.pitch_bend = f32::from(val_u14);
            }
            Some(MidiMessage::ControlChange(_, cc, val7)) => match u8::from(cc) {
                CC_MOD_WHEEL => self.mod_wheel = value7_to_f32(val7),
                CC_VOLUME => self.volume = value7_to_f32(val7),
                CC_VCF_CUTOFF => self.vcf_cutoff = value7_to_f32(val7),
                CC_VCF_RESONANCE => self.vcf_resonance = value7_to_f32(val7),
                CC_PORTAMENTO_TIME => self.portamento_time = value7_to_f32(val7),
                CC_PORTAMENTO_SWITCH => self.portamento_enabled = U7_HALF_SCALE <= u8::from(val7),
                CC_SUSTAIN_SWITCH => self.sustain_enabled = U7_HALF_SCALE <= u8::from(val7),
                CC_ALL_CONTROLLERS_OFF => self.reset_controllers(),
                CC_ALL_NOTES_OFF => {
                    self.held_down_notes.clear();
                    self.gate = false;
                    self.rising_gate = false;
                    self.falling_gate = false;
                }
                _ => (), // ignore all other MIDI CC messages
            },
            _ => (), // ignore all other MIDI messages
        }
    }

    /// `mr.handle_note_on(n, v)` updates the internal state after receiving a note-on message
    fn handle_note_on(&mut self, note: u8, velocity: Value7) {
        self.velocity = value7_to_f32(velocity);

        self.held_down_notes.push(note).ok();

        match self.note_priority {
            // we know that there is at least one element in the vec
            NotePriority::Last => self.note_num = *self.held_down_notes.last().unwrap(),
            NotePriority::High => {
                self.note_num = *self.held_down_notes.iter().max().unwrap();
            }
            NotePriority::Low => {
                self.note_num = *self.held_down_notes.iter().min().unwrap();
            }
        }

        self.gate = true;
        self.falling_gate = false;

        if (self.retrigger_mode == RetriggerMode::AllowRetrigger)
            | (self.held_down_notes.len() == 1)
        {
            self.rising_gate = true;
        }
    }

    /// `mr.handle_note_off()` updates the internal state after receiving a note-off message
    fn handle_note_off(&mut self, note: u8) {
        // delete the note from the list of notes which are held down
        self.held_down_notes.retain(|n| *n != note);

        if self.held_down_notes.is_empty() {
            self.gate = false;
            self.rising_gate = false;
            self.falling_gate = true;
        } else {
            match self.note_priority {
                // we know that there is at least one element in the vec
                NotePriority::Last => self.note_num = *self.held_down_notes.last().unwrap(),
                NotePriority::High => {
                    self.note_num = *self.held_down_notes.iter().max().unwrap();
                }
                NotePriority::Low => {
                    self.note_num = *self.held_down_notes.iter().min().unwrap();
                }
            }
        }
    }

    /// `mr.note_num()` is the current MIDI note number held by the MIDI receiver
    pub fn note_num(&self) -> u8 {
        self.note_num
    }

    /// `mr.pitch_bend()` is the current MIDI pitch-bend value held by the MIDI receiver, in `[-1.0, 1.0]`
    pub fn pitch_bend(&self) -> f32 {
        self.pitch_bend
    }

    /// `mr.velocity()` is the current MIDI velocity value held by the MIDI receiver, in `[-1.0, 1.0]`
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// `mr.mod_wheel()` is the current MIDI mod-wheel value held by the MIDI receiver, in `[0.0, 1.0]`
    pub fn mod_wheel(&self) -> f32 {
        self.mod_wheel
    }

    /// `mr.volume()` is the current MIDI volume value held by the MIDI receiver, in `[0.0, 1.0]`
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// `mr.vcf_cutoff()` is the current MIDI VCF-cutoff value held by the MIDI receiver, in `[0.0, 1.0]`
    pub fn vcf_cutoff(&self) -> f32 {
        self.vcf_cutoff
    }

    /// `mr.vcf_resonance()` is the current MIDI VCF-resonance value held by the MIDI receiver, in `[0.0, 1.0]`
    pub fn vcf_resonance(&self) -> f32 {
        self.vcf_resonance
    }

    /// `mr.portamento_time()` is the current MIDI portamento-time value held by the MIDI receiver, in `[0.0, 1.0]`
    pub fn portamento_time(&self) -> f32 {
        self.portamento_time
    }

    /// `mr.portamento_enabled()` is true if MIDI portamento is currently enabled
    pub fn portamento_enabled(&self) -> bool {
        self.portamento_enabled
    }

    /// `mr.sustain_enabled()` is true if MIDI sustain is currently enabled
    pub fn sustain_enabled(&self) -> bool {
        self.sustain_enabled
    }

    /// `mr.gate()` is true if any MIDI notes are currently being played
    pub fn gate(&self) -> bool {
        self.gate
    }

    /// `mr.rising_gate()` is true if a new note has been played after all notes were lifted. Self clearing.
    pub fn rising_gate(&mut self) -> bool {
        if self.rising_gate {
            self.rising_gate = false;
            true
        } else {
            false
        }
    }

    /// `mr.falling_gate()` is true if all notes have been released after at least one note was played. Self clearing.
    pub fn falling_gate(&mut self) -> bool {
        if self.falling_gate {
            self.falling_gate = false;
            true
        } else {
            false
        }
    }

    /// `mr.set_retrigger_mode(m)` sets the retrigger mode to the given mode `m`
    pub fn set_retrigger_mode(&mut self, mode: RetriggerMode) {
        self.retrigger_mode = mode;
    }

    /// `mr.set_note_priority(p)` sets the note priority to `p`
    pub fn set_note_priority(&mut self, priority: NotePriority) {
        self.note_priority = priority;
    }

    /// `mr.reset_controllers()` resets all implemented MIDI controllers to their default values
    fn reset_controllers(&mut self) {
        self.pitch_bend = 0.0_f32;
        self.mod_wheel = 0.0_f32;
        self.volume = 0.0_f32;
        self.vcf_cutoff = 0.0_f32;
        self.vcf_resonance = 0.0_f32;
        self.portamento_time = 0.0_f32;
        self.portamento_enabled = true;
        self.sustain_enabled = true;
    }
}

/// Retrigger mode is represented here
///
/// Retriggering means that if the user plays a new MIDI note before releasing the last one, a new rising gate will
/// be triggered.
///
/// When retriggering is disabled this is sometimes called "legato" mode, as overlapping notes blend together.
///
/// Classic instruments have used both variations. The MiniMoog does not allow retriggering, while the Arp Odyssey does.
#[derive(PartialEq, Eq)]
pub enum RetriggerMode {
    AllowRetrigger,
    NoRetrigger,
}

/// Note priority is represented here
///
/// When more than one note is played at a time on a monophonic instrument, we need to decide which note takes priority.
///
/// - `Last` priority means that whichever note was played most recently wins
///
/// - `High` priority means that whichever note is highest in pitch wins
///
/// - `Low` priority means that whichever note is lowest in pitch wins
pub enum NotePriority {
    Last,
    High,
    Low,
}

///`value7_to_f32(v)` is the Value7 converted to f32 in `[0.0, 1.0]`
fn value7_to_f32(val7: Value7) -> f32 {
    u8::from(val7) as f32 / 127.0_f32
}

// Common MIDI CC names
const CC_MOD_WHEEL: u8 = 0x01;
const CC_VOLUME: u8 = 0x07;
const CC_EXPRESSION: u8 = 0x0B;
const CC_VCF_CUTOFF: u8 = 0x47;
const CC_VCF_RESONANCE: u8 = 0x4A;
const CC_SUSTAIN_SWITCH: u8 = 0x40;
const CC_PORTAMENTO_SWITCH: u8 = 0x41;
const CC_PORTAMENTO_TIME: u8 = 0x05;
const CC_ALL_CONTROLLERS_OFF: u8 = 0x79;
const CC_ALL_NOTES_OFF: u8 = 0x7B;

// for MIDI CC used as switches values below half scale are considered false and values at-least half scale are true
const U7_HALF_SCALE: u8 = 1 << 6;
