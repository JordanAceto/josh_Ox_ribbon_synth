# Rust Language STM32L412KBU Digital Ribbon Controller

## Features:
- Independent VCO, MODOSC, and VCF attenuators
- Glide control
- Quantizer for VCO only with three modes:
  - QUANTIZE: hard quantization, notes zipper to one another
  - ASSIST: initial finger presses attempt to play in-tune, but sliding is smooth
  - SMOOTH: unquantized smooth ribbon

## MIDI implementation
There is some MIDI functionality baked into the hardware, but not all functions are implemented in software.

It is possible that future improvements will expand on the MIDI functionality.

