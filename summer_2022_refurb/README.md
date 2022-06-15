# Big Summer 2022 Refurb

## Goals
- Redo all the handmade protopboards and rats nest wiring with nice printed pcbs
- Generate good quality documentation
- Add any mods that Ox wants which are feasable
- Maintain the general architecture, but make circuit improvements/updates/changes where appropriate

## Status
Schematic capture and board layout has begun. Everything is still in flux, none of the schematics or pcb layouts are final. The ADSR, VCF/VCA, Sample & Hold, and ModOsc boads are pretty far along, but changes are still expected.

The block diagram and pcb sandbox are to aid in making good interfaces between the boards. Ideally headers will be logically placed for relatively straight wire runs that don't criss cross all over the place.

## Quick notes on the existing circuits:
- the ring mod takes the VCO suboct 2 and the switched MOD OSC as X and Y inputs, should I make the new one different?
- raw envelopes are [0, +10v]

- VCO 
    - range (no ribbon): [9Hz, 6kHz]
    - range (max ribbon): [25Hz, 2kHz]
    - fine tune range: a little more than 1 octave
    - output amplitude: 2.6VPP max, soft-clipped, distortion starts at about 2VPP

- VCF
    - range (no ribbon): [30Hz, 20kHz]
    - could be less range, Josh said the freq knob feels useless for the top 1/4 turn or so
    - separation: about 2.5 octaves in either direction
    - ribbon ctl pots are backwards! (pot 1 controls VCF 2, and vice versa)

- Ribbon
    - range: [2.2V, 8.4V] 

- Modosc
    - range (no ribbon): Lo [88mHz, 8Hz], Mid [2.5Hz, 230Hz], Hi [83Hz, 7kHz]
    - output amplitude: 2.7VPP

- Noise
    - amplitude: 14VPP, mild clipping once in a while

- ADSR attack
    - min time (fast): about 500us
    - min time (slow): about 3ms
    - max time (fast): about 2.75s
    - max time (slow): about 20s