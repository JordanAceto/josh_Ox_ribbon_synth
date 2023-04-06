#![no_std]
#![no_main]

mod board;
mod glide_processor;
mod quantizer;
mod ui;

use synth_utils::{mono_midi_receiver, ribbon_controller};

use crate::{
    board::{AdcPin, Board, Dac8164Channel},
    glide_processor::GlideProcessor,
    quantizer::Quantizer,
    ui::{LevelPot, PitchMode, UiState},
};

use panic_halt as _;

use cortex_m_rt::entry;

const FAST_RIBBON_SAMPLE_RATE: u32 = board::TIM2_FREQ_HZ;
const OUTPUT_UPDATE_SAMPLE_RATE: u32 = board::TIM15_FREQ_HZ;

const RIBBON_PIN: AdcPin = AdcPin::PA4;

const RIBBON_BUFF_CAPACITY: usize =
    ribbon_controller::sample_rate_to_capacity(FAST_RIBBON_SAMPLE_RATE);

// The maximum span of the ribbon, may be attenuated to less than this
const RIBBON_MAX_OCTAVE_SPAN: f32 = 4.0_f32;

#[entry]
fn main() -> ! {
    let mut board = Board::init();
    let mut ui = UiState::new();

    // we need to use the sample rate for both the parameter and argument, if
    // rust support for generic expressions improves then this should be refactored
    let mut ribbon = ribbon_controller::RibbonController::<RIBBON_BUFF_CAPACITY>::new(
        FAST_RIBBON_SAMPLE_RATE as f32,
        15_994.0, // end-to-end resistance of the softpot as measured
        820.4,    // resistance of the series resistor going to vref as measured
    );

    let mut vco_quantizer = Quantizer::new();

    // independent glide processors for each major signal
    let mut glide = [
        GlideProcessor::new(OUTPUT_UPDATE_SAMPLE_RATE),
        GlideProcessor::new(OUTPUT_UPDATE_SAMPLE_RATE),
        GlideProcessor::new(OUTPUT_UPDATE_SAMPLE_RATE),
    ];

    let mut midi_receiver = mono_midi_receiver::MonoMidiReceiver::new(0);

    midi_receiver.set_note_priority(mono_midi_receiver::NotePriority::Last);

    let mut offset_when_finger_pressed_down: f32 = 0.0_f32;

    // small delay to allow the ribbon voltage to settle before beginning
    board.delay_ms(100);

    ui.update(&mut board);

    // index into the array of glide processors so we can rotate which one we update each tick
    let mut glide_idx: usize = 0;

    loop {
        if let Some(b) = board.serial_read() {
            midi_receiver.parse(b)
        }

        // slow timer for updating UI, reading pots and such
        if board.get_tim6_timeout() {
            ui.update(&mut board);

            // Only update the control for 1 glide processor each round, it is a costly call and there is no
            // need to update it super fast. The single glide control is shared by all of the glide processors.
            glide[glide_idx].set_glide(ui.get_glide_ctl());
            glide_idx += 1;
            glide_idx %= glide.len();
        }

        // fast timer for polling the ribbon
        if board.get_tim2_timeout() {
            let raw_adc_val = board.read_adc(RIBBON_PIN);
            ribbon.poll(raw_adc_val);
        }

        // timer to update analog and MIDI outputs
        if board.get_tim15_timeout() {
            // expand the ribbon signal to 1volt/octave range
            let ribbon_as_1v_per_oct = ribbon_to_dac8164_1v_per_oct(ribbon.value());

            // attenuate the ribbon signals with the front panel controls
            let vco_ribbon_contrib = ui.attenuate(ribbon_as_1v_per_oct, LevelPot::Vco);
            let modosc_ribbon_contrib = ui.attenuate(ribbon_as_1v_per_oct, LevelPot::ModOsc);
            let vcf_ribbon_contrib = ui.attenuate(ribbon_as_1v_per_oct, LevelPot::Vcf);

            // only the VCO signal gets quantized
            let quantized_vco_ribbon = vco_quantizer.convert(vco_ribbon_contrib);

            let finger_just_pressed = ribbon.finger_just_pressed();

            // the VCO can be one of three modes
            let vco_ribbon_contrib = match ui.pitch_mode() {
                // hard-quantize and smooth modes are simple to calculate
                PitchMode::HardQuantize => quantized_vco_ribbon.stairstep,
                PitchMode::Smooth => {
                    // a small fudge factor helps keep smooth mode in tune with the other modes
                    let fudge_factor = 0.005;
                    vco_ribbon_contrib - fudge_factor
                }
                // assist mode has more going on
                PitchMode::Assist => {
                    if finger_just_pressed {
                        // When the user first presses down after having lifted their finger record the offset between the
                        // finger position and the center of the note. We'll use this offset to make sure that it plays
                        // a nice in-tune note at first-press.
                        offset_when_finger_pressed_down = quantized_vco_ribbon.fraction;
                        // use the stairstep for the first press for a nice in-tune note
                        quantized_vco_ribbon.stairstep
                    } else {
                        // The user is continuing to press the ribbon and maybe sliding around, use the smooth val but
                        // remove the offset
                        vco_ribbon_contrib - offset_when_finger_pressed_down
                    }
                }
            };

            let midi_1v_per_oct = note_num_to_dac8164_1v_per_oct(midi_receiver.note_num())
                + (midi_receiver.pitch_bend() * 2.0_f32 / 12.0_f32);

            // VCO always gets un-attenuated MIDI note information so it plays in-tune
            let vco_midi_contrib = midi_1v_per_oct;
            // MODOSC and VCF attenuate the MIDI pitch signal with the same knob used to attenuate the ribbon
            let modosc_midi_contrib = ui.attenuate(midi_1v_per_oct, LevelPot::ModOsc);
            let vcf_midi_contrib = ui.attenuate(midi_1v_per_oct, LevelPot::Vcf);

            // apply portamento to each signal
            let final_vco_ribbon = glide[0].process(vco_ribbon_contrib + vco_midi_contrib);
            let final_modosc_ribbon = glide[1].process(modosc_ribbon_contrib + modosc_midi_contrib);
            let final_vcf_ribbon = glide[2].process(vcf_ribbon_contrib + vcf_midi_contrib);

            // set the analog outputs
            board.dac8164_set_vout(final_vco_ribbon, Dac8164Channel::A);
            board.dac8164_set_vout(final_modosc_ribbon, Dac8164Channel::B);
            board.dac8164_set_vout(final_vcf_ribbon, Dac8164Channel::C);

            let mod_wheel_fullscale_volts = 5.0_f32;
            let midi_mod_wheel = midi_receiver.mod_wheel() * mod_wheel_fullscale_volts;
            board.dac8164_set_vout(midi_mod_wheel, Dac8164Channel::D);

            // set the gate high with either the ribbon or MIDI signal
            board.set_gate(ribbon.finger_is_pressing() | midi_receiver.gate());
        }
    }
}

/// `ribbon_to_dac8164_1v_per_oct(r)` is the ribbon value in `[0.0, 1.0]` scaled to 1 volt per octave
fn ribbon_to_dac8164_1v_per_oct(ribb: f32) -> f32 {
    ribb * (RIBBON_MAX_OCTAVE_SPAN + 1.0_f32 / 12.0_f32)
}

/// `note_num_to_dac8164_1v_per_oct(n)` is the note number `n` scaled to 1volt/octave
fn note_num_to_dac8164_1v_per_oct(note_num: u8) -> f32 {
    note_num as f32 / 12.0_f32
}
