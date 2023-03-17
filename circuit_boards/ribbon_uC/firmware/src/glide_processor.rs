use biquad::*;

/// A glide processor for implementing portamento is represented here.
///
/// The terms glide and portamento are used interchangeably.
pub struct GlideProcessor {
    // min and max cutoff frequencies
    min_fc: f32,
    max_fc: f32,

    // sampe rate in hertz
    fs: Hertz<f32>,

    // internal lowpass filter to implement the glide
    lpf: DirectForm1<f32>,

    // cached val to avoid recalculating unnecessarily
    cached_ctl_val: f32,
}

impl GlideProcessor {
    /// `GlideProcessor::new(sr)` is a new glide processor with sample rate `sr`
    pub fn new(sample_rate_hz: u32) -> Self {
        let coeffs = coeffs(sample_rate_hz.hz(), 15.0_f32.hz());

        Self {
            max_fc: 20.0_f32, // just needs to be fast enough to be faster than finger wiggles
            min_fc: 0.3_f32, // adjusted to taste, slow enough that you get some serious glide, but not too slow
            fs: sample_rate_hz.hz(),
            lpf: DirectForm1::<f32>::new(coeffs),
            cached_ctl_val: 0.0_f32,
        }
    }

    /// `gp.set_glide(v)` sets the portamento time for the glide processor to the new value `v`
    ///
    /// # Arguments:
    ///
    /// * `v` - the new value for the glide control, in `[0.0, 1.0]`
    ///
    /// Values near zero are fast, values near 1.0 are slow, similar to how you "turn up" the glide control on a synth
    /// to make the glide time longer.
    ///
    /// This function can be somewhat costly, so don't call it more than necessary
    pub fn set_glide(&mut self, val: f32) {
        let val = val.min(1.0).max(0.0);

        let epsilon = 0.01_f32;

        // don't update the coefficients if you don't need to, it is costly
        if abs_f32(val - self.cached_ctl_val) < epsilon {
            return;
        }

        self.cached_ctl_val = val;

        // convert the unitless [0, 1] input value into a cutoff frequency in the desired range
        let f_range = self.max_fc - self.min_fc;
        let f0 = ((1.0f32 - val) * f_range) + self.min_fc;
        self.lpf.update_coefficients(coeffs(self.fs, f0.hz()))
    }

    /// gp.process(v)` is the value `v` processed by the glide processor, must be called periodically at the sample rate
    pub fn process(&mut self, val: f32) -> f32 {
        self.lpf.run(val)
    }
}

/// `coeffs(fs, f0)` is the lowpass filter coefficients for sample rate `fs` and cutoff frequency `f0`
fn coeffs(fs: Hertz<f32>, f0: Hertz<f32>) -> Coefficients<f32> {
    Coefficients::<f32>::from_params(Type::SinglePoleLowPass, fs, f0, 0.0_f32).unwrap()
}

/// `abs_f32(v)` is |v|
fn abs_f32(v: f32) -> f32 {
    if v < 0.0 {
        -v
    } else {
        v
    }
}
