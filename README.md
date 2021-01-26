# josh_Ox_ribbon_synth
Ribbon synth for Josh Oxford featuring a single VCO with two sub-oscillators, two state-variable filters, ring-modulator, VCA, three ADSR's, and a versatile modulation oscillator.

The state of the documentation is... pretty sad. The synth itself has been complete and working for several years, but the documentation is spotty and incomplete.

Most major components have at least some documentation, excluding the power supply and IO jacks. Each major circuit board has its own directory in this repo, usually with the necessary files to open the project in kicad, and a pdf copy of the schematic of the given board. The power supply is a bog-standard linear supply using common three-terminal regulators.

The VCO and modulation oscillator are fashioned after the "DIY-Dixie" VCO from David Dixon. The ribbon itself is an implementation of the "Appendage Ribbon Controller". The documentation for these is very poor, basically just a copy of the documents from the respective projects.

The sub-oscillators are fashioned after the Scott Stites "Wave Thing" that was discussed on electro-music.com. The sub oscillators are also processed with "Saw Animators" from Electronotes. The second sub-oscillator has an interesting "Harmonic-Pulse" generator built using ideas from Ian Fritz from Electronotes as well.

The filter section is a dual state-variable filter, implemented with SSM2164 quad VCA chips. The final VCA is built with a SSM2164 as well.

The ADSRs are based on a voltage-controlled design by Jurgen Haible from his old website. Voltage-controlled ADSRs were chosen primarily to simplify the panel wiring, since they allow for the potentiometers to simply be strung between a voltage rail and then a single wire per parameter run to the ADSR board. This saves quite a few wires. The gate inputs for the ADSRs and sample-and-hold are buffered by CMOS inverters, in a silly arrangement that could probably be simplified.

The modulation-oscillator, ring-modulator, sample-and-hold, and noise generator are on a circuitboard together. The Modulation oscillator is very similar to the main VCO, both have the same basic core. The modulation-oscillator has its own VCA as well, for dynamic depth modulation. The ring-modulator is built with an LM1496. The sample-and-hold uses an LF398, which also shows up a bunch in the ribbon-controller. The noise-source is a classic reverse-biased NPN with gain, and a "pink-ish" noise filter. 

Each circuit board is built on pad-per-hole prototyping board, and flying wires are soldered between the various boards and the panel mount components. Components on the schematics are mostly unnumbered, so tracing out the circuit takes considerable sluething. I'm very sorry if any poor soul is tasked with maintaining this thing in the future.

If you are a tech and find yourself with this thing on your bench, feel free to reach out and I can try to talk you though some of the questionable design and construction decisions.

## It is guaranteed that there are errors and omissions in the schematics, sorry :(
