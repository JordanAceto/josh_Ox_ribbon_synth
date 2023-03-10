/// A quantizer which converts smooth inputs into stairsteps is represented here.
///
/// Quantizers are used in musical systems to force smoothly changing signals to to take on discrete note values so that
/// the musician can more easily play in-tune. Some hysteresis is applied to prevent chattering around note boundaries.
pub struct Quantizer {
    // save the last conversion for hysteresis purposes
    cached_stairstep: f32,
}

/// A quantizer conversion is represented here.
///
/// Conversions consist of a stairstep portion and fractional portion.
/// The stairstep is the input value converted to a stairstep with as many steps as there are semitones, and the
/// fractional part is the difference between the actual input value and the quantized stairstep.
#[derive(Clone, Copy)]
pub struct Conversion {
    pub stairstep: f32,
    pub fraction: f32,
}

impl Quantizer {
    /// `Quantizer::new()` is a new quantizer.
    pub fn new() -> Self {
        Self {
            cached_stairstep: 0.0f32,
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
    /// * `Conversion` - the converted input as a stairstep and fractional portion
    pub fn convert(&mut self, val: f32) -> Conversion {
        // clamp
        let val = val.min(1.0_f32).max(0.0_f32);

        // center the last val in the middle of its bucket so we can check if the new val is close or far to the center
        // this makes sure we can handle the very low end of the range
        let last_centered_conversion = self.cached_stairstep + HALF_BUCKET_WIDTH;

        // check how far the new val is from the center of the last conversion
        let abs_diff = if val < last_centered_conversion {
            last_centered_conversion - val
        } else {
            val - last_centered_conversion
        };

        // only register a new conversion if the input is far enough away from the last one
        if (HALF_BUCKET_WIDTH + HYSTERESIS) < abs_diff {
            let val_as_int = (val * NUM_SEMITONES as f32) as u8;
            self.cached_stairstep = (val_as_int as f32) / (NUM_SEMITONES as f32);
        }

        let fraction = val - self.cached_stairstep;

        Conversion {
            stairstep: self.cached_stairstep,
            fraction,
        }
    }
}

/// The number of octaves that the quantizer can handle.
const NUM_OCTAVES: u8 = 2;

/// The number of semitones the quantizer can handle.
///
/// The +1 is so you end at an octave instead of a major-7
const NUM_SEMITONES: u8 = NUM_OCTAVES * 12 + 1;

/// The width of each bucket for the semitones.
const BUCKET_WIDTH: f32 = 1.0_f32 / NUM_SEMITONES as f32;

/// 1/2 bucket width
const HALF_BUCKET_WIDTH: f32 = BUCKET_WIDTH / 2.0_f32;

/// Hysteresis provides some noise immunity and prevents oscillations near transition regions.
///
/// Derived empirically, can be adjusted after testing the hardware
const HYSTERESIS: f32 = BUCKET_WIDTH / 10.0_f32;
