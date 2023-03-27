# Rust Language STM32L412KBU Digital Ribbon Controller

## The firmware is installed on an STM32 microcontroller on the ribbon board via SWD header

---

## What it does
- Reads the analog ribbon controller and converts it into position and finger-down information
- Reads incoming MIDI messages and merges MIDI note information with the ribbon control
- Generates analog 1volt/octave and gate signals which can be used to control the VCO, MODOSC, VCF, and ADSRs

---

## Features
- Independent VCO, MODOSC, and VCF attenuators
- Glide control
- Quantizer for VCO only with three modes:
  - QUANTIZE: hard quantization, notes zipper to one another
  - ASSIST: initial finger presses attempt to play in-tune, but sliding is smooth
  - SMOOTH: unquantized smooth ribbon

---

## MIDI implementation
- There is some MIDI functionality baked into the hardware, but it is not complete.
- Same for the software, MIDI input is farther along than MIDI output.
- At this moment, there is no MIDI jack exposed to the outside world, so there is no way for the user to use MIDI.
- It is possible that future improvements will expand on the MIDI functionality. There are internal headers on the ribbon circuit board for future MIDI IO expansion.
