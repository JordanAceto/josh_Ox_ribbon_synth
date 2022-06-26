# Makefile for invoking KiBot documentation and fabrication generation.
# See the .kibot directory for more information.

# For each board you wish to generate documentation for, add the directory
# for that board to the list below.
# There must be exactly 1 KiCad 6 project in each directory listed, inside a
# subdirectory named `kicad_6`.

# The directory structure should look something like this:
# .
# ├─── Makefile (this file)
# │
# ├─── .kibot
# │   ├─── various KiBot files and shell scripts
# │   └───...
# │
# ├─── board_1_dir
# │   ├─── ...
# │   └─── kicad_6
# │       ├── project_1_name.kicad_pcb
# │       └── ...
# │
# ├─── board_2_dir
# │   ├─── ...
# │   └─── kicad_6
# │       ├── project_2_name.kicad_pcb
# │       └── ...
# │
# ├─── ...
# │
# ├─── board_n_dir
# │   └─── ...
# │
# └─── ...

BOARDS = ADSR_board
BOARDS += main_VCO_board
BOARDS += mod_osc_board
BOARDS += ribbon_board
BOARDS += sample_and_hold_board
BOARDS += VCF_VCA_board

# generate documentation for all boards
all:
	./.kibot/doc_gen.sh $(BOARDS)

# delete all documentation
clean:
	./.kibot/cleanup.sh $(BOARDS)

# generate documentation for a single board provided as an argument
board:
	./.kibot/doc_gen.sh $(BOARD)

# delete documentation for a single board provided as an argument
clean_board:
	./.kibot/cleanup.sh $(BOARD)
