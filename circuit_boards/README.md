# Big Summer 2022 Refurb

## Goals
- Redo all the handmade protopboards and rats nest wiring with nice printed pcbs
- Generate good quality documentation
- Add any mods that Ox wants which are feasable
- Maintain the general architecture, but make circuit improvements/updates/changes where appropriate

## Status
All major boards are built and tested. Waiting on next rev of the main VCO board.

## Quick notes on the old circuits:

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
    
## Notes on the test pcb builds:
- ADSR
    - 100 ohms for R411 etc, gate threshold is +3.6V

- VCF/VCA
    - almost done
    - choose Q pot R (18k with 10k pot?)
    - choose separation Rs (390k)
    - choose base freq tune R (68k? not ideal, but I would need another R to -5v to make it perfect)
    - choose zeners (5v1 in now for SVF)

- S&H
    - noise amplitude is a little low
    - you may want to change the comparator positive feedback resistor, 100k -> 1M or something
    - the white/pink switch is too far down for the og panel, oh well

- Mod osc
    - will prob want to change the gain @ U202B
    - I really don't love the rotary-switch/pot setup
        - it would be nice to use the typical SR1712F switches and RD901F pots. SR1712F switches come in 4 and 6 throws, see if you can come up with one more waveshape? Narrow pulse? Random gates?
        - note that this will complicate the switch wiring AND complicate the VCA setup, mebe doable tho?

- VCO
    - major error, when the suboctaves are in the center-off position, there is tremendous animator LFO bleedthrough, will prob need to rethink the switches
    - minor error, the -5v ref net does not reach every node, because I named it two different ways (`-5V_REF` and `-5V REF`), d'oh, can be kludged
    - there could be more test points, I just got lazy
    - it might make more sense to call the saw offset trimmer `SUBOCT n TRI GLITCH` or something?
    - Add a jumper for the ringmod XX input to allow subocts
    - add divider R for PWM LFO square out
    - PWM LFO comparator is wired wrong

- Ribbon
    - I forgor to put labels on the trimpots, oops, there are only two so not the wurst, but darn

- Power supply
    - one of the IDC headers is a wee bit off center, nbd

