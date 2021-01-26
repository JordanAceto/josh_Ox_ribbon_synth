# josh_Ox_ribbon_synth
Ribbon synth for Josh Oxford featuring a single VCO with two sub-oscillators, two state-variable filters, ring-modulator, VCA, three ADSR's, and a versatile modulation oscillator.

The state of the documentation is... pretty sad. The synth itself has been complete for several years, but the documentation is spotty and incomplete.

Most major components have at least some documentation, excluding the power supply and IO jacks. Each major circuit board has its own directory in this repo, usually with the necessary files to open the project in kicad, and a pdf copy of the schematic of the given board. The power supply is a bog-standard linear supply using common three-terminal regulators.

The VCO and modulation oscillator are fashioned after the "DIY-Dixie" VCO from David Dixon. The ribbon itself is an implementation of the "Appendage Ribbon Controller". The documentation for these is very poor, basically just a copy of the documents from the respective projects.

Each circuit board is built on pad-per-hole prototyping board, and flying wires are soldered between the various boards and the panel mount components. Components on the schematics are mostly unnumbered, so tracing out the circuit takes considerable sluething. I'm very sorry if any poor soul is tasked with maintaining this thing in the future.

If you are a tech and find yourself with this thing on your bench, feel free to reach out and I can try to talk you though some of the questionable design and construction decisions.
