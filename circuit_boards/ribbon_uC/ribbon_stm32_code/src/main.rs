#![no_std]
#![no_main]

mod board;
mod glide_processor;
mod midi_note_converter;
mod midi_transmitter;
mod quantizer;
mod ribbon_controller;
mod ui;

use panic_halt as _;

use crate::board::{AdcPin, Board, Dac8164Channel};
use crate::glide_processor::GlideProcessor;
use crate::quantizer::Quantizer;
use crate::ribbon_controller::RibbonController;
use crate::ui::{LevelPot, PitchMode, UiState};

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    const MAIN_SAMPLE_RATE_HZ: u32 = board::TIM2_FREQ_HZ;

    const RIBBON_PIN: AdcPin = AdcPin::PA4;

    let mut board = Board::init();
    let mut ui = UiState::new();

    // we need to use the sample rate for both the parameter and argument, if
    // rust support for generic expressions improves then this should be refactored
    let mut ribbon = RibbonController::<
        { ribbon_controller::sample_rate_to_capacity(MAIN_SAMPLE_RATE_HZ) },
    >::new(MAIN_SAMPLE_RATE_HZ);

    let mut vco_quantizer = Quantizer::new();

    // independent glide processors for each major signal
    let mut glide = [
        GlideProcessor::new(MAIN_SAMPLE_RATE_HZ),
        GlideProcessor::new(MAIN_SAMPLE_RATE_HZ),
        GlideProcessor::new(MAIN_SAMPLE_RATE_HZ),
    ];

    // small delay to allow the ribbon voltage to settle before beginning
    board.delay_ms(100);

    ui.update(&mut board);

    glide
        .iter_mut()
        .for_each(|g| g.set_glide(ui.get_glide_ctl()));

    let mut pitch_mode = ui.pitch_mode();

    // used in ASSIST mode to help notes be in-tune when you first press down
    let mut offset_when_finger_pressed_down: f32 = 0.0f32;

    // index into the array of glide processors so we can rotate which one we update each tick
    let mut glide_idx: usize = 0;

    loop {
        // slow timer for updating UI, reading pots and such
        if board.get_tim6_timeout() {
            ui.update(&mut board);

            // Only update the control for 1 glide processor each round, it is a costly call and there is no
            // need to update it super fast. The single glide control is shared by all of the glide processors.
            // static mut COUNTER: usize = 0;
            glide[glide_idx].set_glide(ui.get_glide_ctl());
            glide_idx += 1;
            glide_idx %= glide.len();

            pitch_mode = ui.pitch_mode();
        }

        // fast timer for updating the ribbon and gate
        if board.get_tim2_timeout() {
            let raw_adc_val = board.read_adc(RIBBON_PIN);
            ribbon.poll(raw_adc_val);

            let vco_ribbon = ui.attenuate(ribbon.value(), LevelPot::Vco);
            let modosc_ribbon = ui.attenuate(ribbon.value(), LevelPot::ModOsc);
            let vcf_ribbon = ui.attenuate(ribbon.value(), LevelPot::Vcf);

            let quantized_vco_ribbon = vco_quantizer.convert(vco_ribbon);

            // the VCO can be one of three modes
            let final_vco_ribbon = match pitch_mode {
                // hard-quantize and smooth modes are simple to calculate
                PitchMode::HardQuantize => quantized_vco_ribbon.stairstep,
                PitchMode::Smooth => {
                    let fudge_factor = 0.015;
                    vco_ribbon - fudge_factor
                }

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

            let final_vco_ribbon = glide[0].process(final_vco_ribbon);
            let final_modosc_ribbon = glide[1].process(modosc_ribbon);
            let final_vcf_ribbon = glide[2].process(vcf_ribbon);

            board.dac8164_write(final_vco_ribbon, Dac8164Channel::A);
            board.dac8164_write(final_modosc_ribbon, Dac8164Channel::B);
            board.dac8164_write(final_vcf_ribbon, Dac8164Channel::C);
            // board.dac8164_write(aux_cv, Dac8164Channel::D); // currently unused aux CV output

            board.set_gate(ribbon.finger_is_pressing());
        }

        // TODO: implement MIDI IO
    }
}
