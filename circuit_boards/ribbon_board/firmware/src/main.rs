#![no_std]
#![no_main]

mod board;
mod glide_processor;
mod midi_transmitter;
mod quantizer;
mod ribbon_controller;
mod ui;

use crate::{
    board::{AdcPin, Board, Dac8164Channel},
    glide_processor::GlideProcessor,
    midi_transmitter::MidiTransmitter,
    quantizer::Quantizer,
    ribbon_controller::RibbonController,
    ui::{LevelPot, PitchMode, UiState},
};

use midi_convert::midi_types::MidiMessage;

use panic_halt as _;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    const FAST_RIBBON_SAMPLE_RATE: u32 = board::TIM2_FREQ_HZ;
    const OUTPUT_UPDATE_SAMPLE_RATE: u32 = board::TIM15_FREQ_HZ;

    const RIBBON_PIN: AdcPin = AdcPin::PA4;

    let mut board = Board::init();
    let mut ui = UiState::new();

    // we need to use the sample rate for both the parameter and argument, if
    // rust support for generic expressions improves then this should be refactored
    let mut ribbon = RibbonController::<
        { ribbon_controller::sample_rate_to_capacity(FAST_RIBBON_SAMPLE_RATE) },
    >::new(FAST_RIBBON_SAMPLE_RATE);

    let mut vco_quantizer = Quantizer::new();

    // independent glide processors for each major signal
    let mut glide = [
        GlideProcessor::new(OUTPUT_UPDATE_SAMPLE_RATE),
        GlideProcessor::new(OUTPUT_UPDATE_SAMPLE_RATE),
        GlideProcessor::new(OUTPUT_UPDATE_SAMPLE_RATE),
    ];

    // TODO: we may eventually include a way to set the MIDI channel, there is a hardware switch to support 16 channels
    let midi_channel = 0;
    let mut midi_transmitter = MidiTransmitter::new();

    // small delay to allow the ribbon voltage to settle before beginning
    board.delay_ms(100);

    ui.update(&mut board);

    let mut pitch_mode = ui.pitch_mode();

    // used in ASSIST mode to help notes be in-tune when you first press down
    let mut offset_when_finger_pressed_down: f32 = 0.0f32;

    // index into the array of glide processors so we can rotate which one we update each tick
    let mut glide_idx: usize = 0;

    // keep track of conversions so we don't write mode MIDI data than needed if nothing changed
    let mut last_midi_note_sent = 0;
    let mut last_pitch_bend = 0.0_f32;

    loop {
        // slow timer for updating UI, reading pots and such
        if board.get_tim6_timeout() {
            ui.update(&mut board);

            // Only update the control for 1 glide processor each round, it is a costly call and there is no
            // need to update it super fast. The single glide control is shared by all of the glide processors.
            glide[glide_idx].set_glide(ui.get_glide_ctl());
            glide_idx += 1;
            glide_idx %= glide.len();

            pitch_mode = ui.pitch_mode();
        }

        // fast timer for polling the ribbon
        if board.get_tim2_timeout() {
            let raw_adc_val = board.read_adc(RIBBON_PIN);
            ribbon.poll(raw_adc_val);
        }

        // timer to update analog and MIDI outputs
        if board.get_tim15_timeout() {
            let vco_ribbon = ui.attenuate(ribbon.value(), LevelPot::Vco);
            let modosc_ribbon = ui.attenuate(ribbon.value(), LevelPot::ModOsc);
            let vcf_ribbon = ui.attenuate(ribbon.value(), LevelPot::Vcf);

            let quantized_vco_ribbon = vco_quantizer.convert(vco_ribbon);

            // these are self clearing and we want to read them a couple times
            let finger_just_pressed = ribbon.finger_just_pressed();
            let finger_just_released = ribbon.finger_just_released();

            // the VCO can be one of three modes
            let final_vco_ribbon = match pitch_mode {
                // hard-quantize and smooth modes are simple to calculate
                PitchMode::HardQuantize => quantized_vco_ribbon.stairstep,
                PitchMode::Smooth => {
                    // a small fudge factor helps keep smooth mode in tune with the other modes
                    let fudge_factor = 0.005;
                    vco_ribbon - fudge_factor
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
                        vco_ribbon - offset_when_finger_pressed_down
                    }
                }
            };

            // apply portamento to each signal
            let final_vco_ribbon = glide[0].process(final_vco_ribbon);
            let final_modosc_ribbon = glide[1].process(modosc_ribbon);
            let final_vcf_ribbon = glide[2].process(vcf_ribbon);

            // set the analog outputs
            board.dac8164_set_vout(
                ribbon_to_dac8164_1v_per_oct(final_vco_ribbon),
                Dac8164Channel::A,
            );
            board.dac8164_set_vout(
                ribbon_to_dac8164_1v_per_oct(final_modosc_ribbon),
                Dac8164Channel::B,
            );
            board.dac8164_set_vout(
                ribbon_to_dac8164_1v_per_oct(final_vcf_ribbon),
                Dac8164Channel::C,
            );
            // board.dac8164_write(aux_cv, Dac8164Channel::D); // currently unused aux CV output

            board.set_gate(ribbon.finger_is_pressing());

            // re-quantize the scaled/processed/portamento'd VCO ribbon signal so we can convert it into a MIDI message
            let this_midi_conversion = vco_quantizer.convert(final_vco_ribbon);

            // Each round there may be between zero and three MIDI messages sent:
            //
            // 1) a note-on message if the user just pressed the ribbon or if they slid into a new note
            // 2) one or two note-off messages if the user just released the ribbon or if they slid into a new note
            // 3) a pitch bend message if the user is pressing the ribbon and the value has changed since last time
            if finger_just_pressed {
                midi_transmitter.push(MidiMessage::NoteOn(
                    midi_channel.into(),
                    this_midi_conversion.note_num.into(),
                    127.into(),
                ));
                last_midi_note_sent = this_midi_conversion.note_num;
            } else if ribbon.finger_is_pressing()
                && this_midi_conversion.note_num != last_midi_note_sent
            {
                midi_transmitter.push(MidiMessage::NoteOn(
                    midi_channel.into(),
                    this_midi_conversion.note_num.into(),
                    127.into(),
                ));
                midi_transmitter.push(MidiMessage::NoteOff(
                    midi_channel.into(),
                    last_midi_note_sent.into(),
                    0.into(),
                ));
                last_midi_note_sent = this_midi_conversion.note_num;
            } else if finger_just_released {
                midi_transmitter.push(MidiMessage::NoteOff(
                    midi_channel.into(),
                    this_midi_conversion.note_num.into(),
                    0.into(),
                ));
                if this_midi_conversion.note_num != last_midi_note_sent {
                    midi_transmitter.push(MidiMessage::NoteOff(
                        midi_channel.into(),
                        last_midi_note_sent.into(),
                        0.into(),
                    ));
                }
            }

            // use the fractional value from the quantizer to calculate the pitch bend that nudges the MIDI note
            // to match the analog voltages generated
            // MIDI pitch bend is usually set to 2 semitones, the extra divide-by-two avoids overshooting
            let this_pitch_bend =
                this_midi_conversion.fraction / (quantizer::BUCKET_WIDTH * 2.0_f32);

            if last_pitch_bend != this_pitch_bend {
                midi_transmitter.push(MidiMessage::PitchBendChange(
                    midi_channel.into(),
                    this_pitch_bend.into(),
                ));
                last_pitch_bend = this_pitch_bend;
            }

            // send any MIDI messages, the queue might be empty but that is fine
            midi_transmitter.send_queue(&mut board);
        }
    }
}

/// The maximum final output voltage for a ribbon signal
const QUANTIZED_RIBBON_VMAX: f32 = (quantizer::NUM_SEMITONES as f32) / 12.0_f32;

/// `ribbon_to_dac8164_1v_per_oct(r)` is the ribbon value in `[0.0, 1.0]` scaled to 1 volt per octave
fn ribbon_to_dac8164_1v_per_oct(ribb: f32) -> f32 {
    ribb * QUANTIZED_RIBBON_VMAX
}
