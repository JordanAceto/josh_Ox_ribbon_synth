#![no_std]
#![no_main]

mod board;
mod midi_note_converter;
mod midi_transmitter;
mod quantizer;
mod ribbon_controller;
mod ui;

use panic_halt as _;

use crate::board::{AdcPin, Board, Dac8164Channel};
use crate::quantizer::Quantizer;
use crate::ribbon_controller::RibbonController;
use crate::ui::{LevelPot, PitchMode, UiState};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    const MAIN_SAMPLE_RATE: u32 = board::TIM2_FREQ_HZ;
    const RIBBON_PIN: AdcPin = AdcPin::PA4;

    let mut board = Board::init();
    let mut ui = UiState::new();

    // we need to use the sample rate for both the parameter and argument, if
    // rust support for generic expressions improves then this should be refactored
    let mut ribbon = RibbonController::<
        { ribbon_controller::sample_rate_to_capacity(MAIN_SAMPLE_RATE) },
    >::new(MAIN_SAMPLE_RATE);

    let mut vco_quantizer = Quantizer::new();

    let mut offset_when_finger_pressed_down: f32 = 0.0f32;

    // small delay to allow the ribbon voltage to settle before beginning
    board.delay_ms(100);

    loop {
        if board.get_tim2_timeout() {
            let raw_adc_val = board.read_adc(RIBBON_PIN);
            ribbon.poll(raw_adc_val);

            ui.update(&mut board);

            let scaled_vco_ribbon = ui.attenuate(ribbon.value(), LevelPot::Vco);
            let scaled_modosc_ribbon = ui.attenuate(ribbon.value(), LevelPot::ModOsc);
            let scaled_vcf_ribbon = ui.attenuate(ribbon.value(), LevelPot::Vcf);

            let quantized_vco_ribbon = vco_quantizer.convert(scaled_vco_ribbon);

            // the VCO can be one of three modes
            let final_vco_ribbon = match ui.pitch_mode() {
                // hard-quantize and smooth modes are simple to calculate
                PitchMode::HardQuantize => quantized_vco_ribbon.stairstep,
                PitchMode::Smooth => scaled_vco_ribbon,

                // assist mode is trickier
                PitchMode::Assist => {
                    if ribbon.finger_just_pressed() {
                        // When the user first presses down after having lifted their finger record the offset between the
                        // finger position and the center of the note. We'll use this offset to make sure that it plays
                        // a nice in-tune note at first-press.
                        offset_when_finger_pressed_down = quantized_vco_ribbon.fraction;
                        quantized_vco_ribbon.stairstep
                    } else {
                        // The user is continuing to press the ribbon and maybe sliding around
                        // scaled_vco_ribbon - offset_when_finger_pressed_down
                        quantized_vco_ribbon.stairstep + quantized_vco_ribbon.fraction
                            - offset_when_finger_pressed_down
                    }
                }
            };

            board.dac8164_write(final_vco_ribbon, Dac8164Channel::A);
            board.dac8164_write(scaled_modosc_ribbon, Dac8164Channel::B);
            board.dac8164_write(scaled_vcf_ribbon, Dac8164Channel::C);
            // board.dac8164_write(aux_cv, Dac8164Channel::D); // currently unused aux CV output

            board.set_gate(ribbon.finger_is_pressing());
        }

        // TODO: implement MIDI IO
    }
}
