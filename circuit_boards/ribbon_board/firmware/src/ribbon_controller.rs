use heapless::HistoryBuffer;

/// A synthesizer ribbon controller is represented here.
///
/// Users play the ribbon by sliding their finger up and down a resistive track wired as a voltage divider.
///
/// We can detect when a user is pressing the ribbon, and where along the track they are pressing with this module.
///
/// The position of the user's finger on the ribbon is represented as a number. The farther to the right the user is
/// pressing the larger the number. The position value is retained even when the user lifts their finger off of the
/// ribbon, similar to a sample-and-hold system. Some averaging is done to smooth out the raw readings and reduce the
/// influence of spurious inputs.
///
/// Whether or not the user is pressing on the ribbon is represented as a boolean  signal.
///
/// The position value and finger-down signals are then typically used as control signals for other modules, such as
/// oscillators, filters, and amplifiers.
///
/// # Inputs
///
/// * Samples are fed into the ribbon controller
///
/// # Outputs
///
/// * The average of the most recent samples representing the position of the user's finger on the ribbon
///
/// * Boolean signals related to the user's finger presses
pub struct RibbonController<const BUFFER_CAPACITY: usize> {
    /// The current position value of the ribbon
    current_val: f32,

    /// The current gate value of the ribbon
    finger_is_pressing: bool,

    /// True iff the gate is rising after being low
    finger_just_pressed: bool,

    /// True iff the gate is falling after being high
    finger_just_released: bool,

    /// An internal buffer for storing and averaging samples as they come in via the `poll` method
    buff: HistoryBuffer<f32, BUFFER_CAPACITY>,

    /// The number of samples to ignore when the user initially presses their finger
    num_to_ignore_up_front: usize,

    /// The number of the most recent sampes to discard
    num_to_discard_at_end: usize,

    /// The number of samples revieved since the user pressed their finger down
    ///
    /// Resets when the user lifts their finger
    num_samples_recieved: usize,

    /// The number of samples actually written to the buffer
    ///
    /// Resets when the user lifts their finger
    num_samples_written: usize,

    /// The sample rate in hertz, ribbon must be polled at this frequency
    _sample_rate_hz: u32,
}

impl<const BUFFER_CAPACITY: usize> RibbonController<BUFFER_CAPACITY> {
    /// `Ribbon::new(sr)` is a new Ribbon controller with sample rate `sr`
    pub fn new(sample_rate_hz: u32) -> Self {
        Self {
            current_val: 0.0_f32,
            finger_is_pressing: false,
            finger_just_pressed: false,
            finger_just_released: false,
            buff: HistoryBuffer::new(),
            num_to_ignore_up_front: ((sample_rate_hz * RIBBON_FALL_TIME_USEC) / 1_000_000) as usize,
            num_to_discard_at_end: ((sample_rate_hz * RIBBON_RISE_TIME_USEC) / 1_000_000) as usize,
            num_samples_recieved: 0,
            num_samples_written: 0,
            _sample_rate_hz: sample_rate_hz,
        }
    }

    /// `rib.poll(raw_adc_value)` updates the ribbon controller by polling the raw ADC signal.
    ///
    /// # Arguments
    ///
    /// * `raw_adc_value` - the raw ADC signal to poll, represents the finger position on the ribbon
    ///
    /// The ribbon must be updated periodically at the chosen sample rate held by the structure. It is required that a
    /// constant stream of ADC samples will be fed into the ribbon by calling this method at the correct sample rate.
    pub fn poll(&mut self, raw_adc_value: f32) {
        let user_is_pressing_ribbon = raw_adc_value < FINGER_PRESS_HIGH_BOUNDARY;

        if user_is_pressing_ribbon {
            self.num_samples_recieved += 1;
            self.num_samples_recieved = self.num_samples_recieved.min(self.num_to_ignore_up_front);
        } else {
            // if this flag is true right now then they must have just lifted their finger
            if self.finger_is_pressing {
                self.num_samples_recieved = 0;
                self.num_samples_written = 0;
                self.finger_just_released = true;
                self.finger_is_pressing = false;
            }
        }

        // only start adding samples to the buffer after we've ignored a few potentially spurious initial samples
        if self.num_to_ignore_up_front <= self.num_samples_recieved {
            self.buff.write(raw_adc_value);

            self.num_samples_written += 1;
            self.num_samples_written = self.num_samples_written.min(self.buff.capacity());

            // is the buffer full?
            if self.num_samples_written == self.buff.capacity() {
                let num_to_take = self.buff.capacity() - self.num_to_discard_at_end;

                // take the average of the most recent samples, minus a few of the very most recent ones which might be
                // shooting up towards full scale when the user lifts their finger
                self.current_val = self.buff.oldest_ordered().take(num_to_take).sum::<f32>()
                    / (num_to_take as f32);

                // if this flag is false right now then they must have just pressed their finger down
                if !self.finger_is_pressing {
                    self.finger_just_pressed = true;
                    self.finger_is_pressing = true;
                }
            }
        }
    }

    /// `rib.value()` is the current position value of the ribbon in `[0.0, 1.0]`
    ///
    /// If the user's finger is not pressing on the ribbon, the last valid value before they lifted their finger
    /// is returned.
    pub fn value(&self) -> f32 {
        // scale the value back to full scale since we loose a tiny bit of range to the high-boundary
        self.current_val / FINGER_PRESS_HIGH_BOUNDARY
    }

    /// `rib.finger_is_pressing()` is `true` iff the user is pressing on the ribbon.
    pub fn finger_is_pressing(&self) -> bool {
        self.finger_is_pressing
    }

    /// `rib.finger_just_pressed()` is `true` iff the user has just pressed the ribbon after having not touched it.
    ///
    /// Self clearing
    pub fn finger_just_pressed(&mut self) -> bool {
        if self.finger_just_pressed {
            self.finger_just_pressed = false;
            true
        } else {
            false
        }
    }

    /// `rib.finger_just_released()` is `true` iff the user has just lifted their finger off the ribbon.
    ///
    /// Self clearing
    pub fn finger_just_released(&mut self) -> bool {
        if self.finger_just_released {
            self.finger_just_released = false;
            true
        } else {
            false
        }
    }
}

/// The end-to-end resistance of the softpot ribbon, in ohms
///
/// If you choose a different sensor, change this to match the resistance of your ribbon membrane
const SOFTPOT_R: f32 = 20_000.;

/// The resistance of the series resistor between the softpot and the 3.3v power supply, in ohms
const UPPER_R: f32 = 820.;

/// Samples below this value indicate that there is a finger pressed down on the ribbon.
///
/// The value must be in [0.0, +1.0], and represents the fraction of the ADC reading which counts as a finger press.
///
/// The exact value depends on the resistor chosen that connects the top of the ribbon to the positive voltage
/// reference. We "waste" a little bit of the voltage range of the ribbon as a dead-zone so we can clearly detect when
/// the user is pressing the ribbon or not.
const FINGER_PRESS_HIGH_BOUNDARY: f32 = 1.0 - (UPPER_R / (UPPER_R + SOFTPOT_R));

/// The approximate measured time it takes for the ribbon to settle on a low value after the user presses their finger.
///
/// We want to ignore samples taken while the ribbon is settling during a finger-press value.
///
/// Rounded up a bit from the actual measured value, better to take a little extra time than to include bad input.
const RIBBON_FALL_TIME_USEC: u32 = 1_000;

/// The approximate measured time it takes the ribbon to rise to the pull-up value after releasing your finger.
///
/// We want to ignore samples that are taken while the ribbon is shooting up towards full scale after lifting a finger.
///
/// Rounded up a bit from the actual measured value, better to take a little extra time than to include bad input.
const RIBBON_RISE_TIME_USEC: u32 = 2_000;

/// The minimum time required to capture a reading
///
/// Ideally several times longer than the sum of the RISE and FALL times
const MIN_CAPTURE_TIME_USEC: u32 = (RIBBON_FALL_TIME_USEC + RIBBON_RISE_TIME_USEC) * 5;

/// `sample_rate_to_capacity(sr_hz)` is the calculated capacity needed for the internal buffer based on the sample rate.
///
/// Const function allows us to use the result of this expression as a generic argument. If rust support for generic
/// expressions improves, this function could be refactored out.
///
/// The capacity needs space for the main samples that we will actually care about, as well as room for the most
/// recent samples to discard. This is to avoid including spurious readings in the average.
pub const fn sample_rate_to_capacity(sample_rate_hz: u32) -> usize {
    // can't use floats in const function yet
    let num_main_samples_to_care_about =
        ((sample_rate_hz * MIN_CAPTURE_TIME_USEC) / 1_000_000) as usize;
    let num_to_discard_at_end = ((sample_rate_hz * RIBBON_RISE_TIME_USEC) / 1_000_000) as usize;

    // +1 at the end just because we'd rather have one-too-many than to truncate down and have one-too-few
    num_main_samples_to_care_about + num_to_discard_at_end + 1
}
