use crate::board::{AdcPin, Board, Switch3wayState};

/// The user interface is represented here (i.e. the front panel pots and switches that the user interacts with)
pub struct UiState {
    pitch_mode: PitchMode,

    vco_lev: f32,
    modosc_lev: f32,
    vcf_lev: f32,

    glide_lev: f32,
}

// There are three modes for the ribbon pitch information
#[derive(Clone, Copy)]
pub enum PitchMode {
    HardQuantize,
    Assist,
    Smooth,
}

#[derive(Clone, Copy)]
pub enum LevelPot {
    VCO,
    MODOSC,
    VCF,
}

impl UiState {
    /// `UiState::new()` is a new UI state initialized to default values.
    pub fn new() -> Self {
        Self {
            pitch_mode: PitchMode::Smooth,
            vco_lev: 0.0_f32,
            modosc_lev: 0.0_f32,
            vcf_lev: 0.0_f32,
            glide_lev: 0.0_f32,
        }
    }

    /// `ui.update()` updates the UI state by reading and storing the panel control user inputs.
    ///
    /// It is expected to call this task periodically, but it can be updated
    /// relatively slowly. Only potentiometer and switch positions are updated, so
    /// 10 or more times per second should be fine.
    pub fn update(&mut self, board: &mut Board) {
        self.pitch_mode = match board.read_mode_switch() {
            Switch3wayState::UP => PitchMode::HardQuantize,
            Switch3wayState::MIDDLE => PitchMode::Assist,
            Switch3wayState::DOWN => PitchMode::Smooth,
        };

        self.vco_lev = board.read_adc(AdcPin::PA3);
        self.modosc_lev = board.read_adc(AdcPin::PA2);
        self.vcf_lev = board.read_adc(AdcPin::PA1);
        self.glide_lev = board.read_adc(AdcPin::PA0);
    }

    /// ui.scale(v, c)` scales the input value `v` by the position of the potentiometer control `c`
    ///
    /// # Arguments:
    ///
    /// * `val` - the value to scale
    ///
    /// * `control` - the enumerated panel control to scale the value with
    pub fn scale(&self, val: f32, control: LevelPot) -> f32 {
        match control {
            LevelPot::VCO => val * self.vco_lev,
            LevelPot::MODOSC => val * self.modosc_lev,
            LevelPot::VCF => val * self.vcf_lev,
        }
    }

    /// `ui.pitch_mode()` is the current enumerated pitch mode, as set by the panel mount switch
    pub fn pitch_mode(&self) -> PitchMode {
        self.pitch_mode
    }
}
