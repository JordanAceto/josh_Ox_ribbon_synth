use crate::board::{AdcPin, Board, Switch3wayState};

/// The user interface is represented here (i.e. the front panel pots and switches that the user interacts with)
pub struct UiState {
    pitch_mode: PitchMode,

    vco_lev: f32,
    modosc_lev: f32,
    vcf_lev: f32,
    delay_lev: f32,
}

/// There are three modes for the ribbon pitch information
#[derive(Clone, Copy)]
pub enum PitchMode {
    HardQuantize,
    Assist,
    Smooth,
}

/// Each main ribbon signal has its own attenuator
#[derive(Clone, Copy)]
pub enum LevelPot {
    Vco,
    ModOsc,
    Vcf,
    Delay,
}

impl UiState {
    /// `UiState::new()` is a new UI state initialized to default values.
    pub fn new() -> Self {
        Self {
            pitch_mode: PitchMode::Smooth,
            vco_lev: 0.0_f32,
            modosc_lev: 0.0_f32,
            vcf_lev: 0.0_f32,
            delay_lev: 0.0_f32,
        }
    }

    /// `ui.update()` updates the UI state by reading and storing the panel control user inputs.
    ///
    /// It is required to periodically call this function to updat the state of the UI controls. Since these controls
    /// are manually adjusted by the user, they don't need to be updated very fast, just fast enough that they don't
    /// feel sluggish to the user.
    pub fn update(&mut self, board: &mut Board) {
        self.pitch_mode = match board.read_mode_switch() {
            Switch3wayState::Up => PitchMode::HardQuantize,
            Switch3wayState::Middle => PitchMode::Assist,
            Switch3wayState::Down => PitchMode::Smooth,
        };

        // the level pots are center-detent so we can easily dial in exactly midway
        self.vco_lev = apply_midpoint_dead_zone(board.read_adc(AdcPin::PA3));
        self.modosc_lev = apply_midpoint_dead_zone(board.read_adc(AdcPin::PA2));
        self.vcf_lev = apply_midpoint_dead_zone(board.read_adc(AdcPin::PA1));
        self.delay_lev = apply_midpoint_dead_zone(board.read_adc(AdcPin::PA0))
    }

    /// `ui.attenuate(v, c)` scales the input value `v` by the position of the front panel potentiometer `c`
    ///
    /// # Arguments:
    ///
    /// * `val` - the value to scale
    ///
    /// * `control` - the enumerated panel control to scale the value with
    ///
    /// # Returns:
    ///
    /// * `val` attenuated by the given control. If the panel control is turned CCW then turn `val` down, if it's
    /// turned CW then turn `val` up.
    pub fn attenuate(&self, val: f32, control: LevelPot) -> f32 {
        match control {
            LevelPot::Vco => val * self.vco_lev,
            LevelPot::ModOsc => val * self.modosc_lev,
            LevelPot::Vcf => val * self.vcf_lev,
            LevelPot::Delay => val * self.delay_lev,
        }
    }

    /// `ui.pitch_mode()` is the current enumerated pitch mode, as set by the panel mount switch
    pub fn pitch_mode(&self) -> PitchMode {
        self.pitch_mode
    }
}

/// `apply_midpoint_dead_zone(v)` is the value `v` with a small dead zone in the center of the range
///
/// This means that for a small portion near the middle of the range changes will have no effect.
/// The is to make it easier to hit the midpoint of center-detent potiometers.
///
/// # Arguments:
///
/// * `val` - the value to apply dead zone to, must be in `[0.0, 1.0]`
fn apply_midpoint_dead_zone(val: f32) -> f32 {
    const DEAD_ZONE_WIDTH: f32 = 0.1_f32;
    const MIDPOINT: f32 = 0.5_f32;
    const DEAD_ZONE_START: f32 = MIDPOINT - DEAD_ZONE_WIDTH / 2.0_f32;
    const DEAD_ZONE_END: f32 = MIDPOINT + DEAD_ZONE_WIDTH / 2.0_f32;
    const SLOPE: f32 = MIDPOINT / DEAD_ZONE_START;

    if val < DEAD_ZONE_START {
        SLOPE * val
    } else if val <= DEAD_ZONE_END {
        // it's in the deadzone
        MIDPOINT
    } else {
        // it must be past the deadzone end
        SLOPE * (val - DEAD_ZONE_END) + MIDPOINT
    }
}
