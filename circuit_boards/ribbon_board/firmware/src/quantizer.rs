/// A quantizer which converts smooth inputs into stairsteps is represented here.
///
/// Quantizers are used in musical systems to force smoothly changing signals to to take on discrete note values so that
/// the musician can more easily play in-tune. Some hysteresis is applied to prevent chattering around note boundaries.
pub struct Quantizer {
    // save the last conversion for hysteresis purposes
    cached_conversion: Conversion,
}

/// A quantizer conversion is represented here.
///
/// Conversions consist of a stairstep portion and fractional portion.
/// The stairstep is the input value converted to a stairstep with as many steps as there are semitones, and the
/// fractional part is the difference between the actual input value and the quantized stairstep.
///
/// The stairstep will always be positive and in `[0.0, 1.0]`, the fraction may be positive or negative.
/// The stairstep plus the fraction will get us back to the original input value.
///
/// The integer note number is also included.
#[derive(Clone, Copy)]
pub struct Conversion {
    pub note_num: u8,
    pub stairstep: f32,
    pub fraction: f32,
}

impl Conversion {
    pub fn default() -> Self {
        Self {
            note_num: 0,
            stairstep: 0.0,
            fraction: 0.0,
        }
    }
}

impl Quantizer {
    /// `Quantizer::new()` is a new quantizer.
    pub fn new() -> Self {
        Self {
            cached_conversion: Conversion::default(),
        }
    }

    /// `qn.convert(val)` is the quantized version of the input value.
    ///
    /// The input is split into a stairstep component and fractional component.
    ///
    /// # Arguments
    ///
    /// * `val` - the value to quantize, in `[0.0, +1.0]`
    ///
    /// # Returns
    ///
    /// * `Conversion` - the input split into a stairstep and fractional portion
    pub fn convert(&mut self, val: f32) -> Conversion {
        // clamp
        let val = val.min(1.0_f32).max(0.0_f32);

        // check how far the new val is from the center of the last conversion
        let abs_diff = if val < self.cached_conversion.stairstep {
            self.cached_conversion.stairstep - val
        } else {
            val - self.cached_conversion.stairstep
        };

        // only register a new conversion if the input is far enough away from the last one
        if HYSTERESIS < abs_diff {
            let val_as_int = (val * NUM_SEMITONES as f32) as u8;

            self.cached_conversion.note_num = val_as_int;
            self.cached_conversion.stairstep = (val_as_int as f32) / (NUM_SEMITONES as f32);
        }

        self.cached_conversion.fraction = val - self.cached_conversion.stairstep;

        self.cached_conversion
    }
}

/// The number of octaves that the quantizer can handle.
pub const NUM_OCTAVES: u8 = 4;

/// The number of semitones the quantizer can handle.
///
/// The +1 is so you end at an octave instead of a major-7
pub const NUM_SEMITONES: u8 = NUM_OCTAVES * 12 + 1;

/// The width of each bucket for the semitones.
pub const BUCKET_WIDTH: f32 = 1.0_f32 / NUM_SEMITONES as f32;

/// Hysteresis provides some noise immunity and prevents oscillations near transition regions.
///
/// Derived empirically, can be adjusted after testing the hardware
const HYSTERESIS: f32 = BUCKET_WIDTH * 0.51_f32;
