# Makefile for invoking KiBot documentation and fabrication generation.

# users must create a file named boards.txt with the names of each board
# they wish to generate documentation for
BOARDS := $(shell cat ../boards.txt)

# generate documentation for all boards
all:
	./doc_gen.sh $(BOARDS)

# delete all documentation
clean:
	./cleanup.sh $(BOARDS)

# generate documentation for a single board provided as an argument
board:
	./doc_gen.sh $(BOARD)

# delete documentation for a single board provided as an argument
clean_board:
	./cleanup.sh $(BOARD)
