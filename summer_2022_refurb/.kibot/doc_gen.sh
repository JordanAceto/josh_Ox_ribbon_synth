#!/bin/bash

cd ../

boards=("$@")

for board in "${boards[@]}";
do
    echo "Generating documentation for $board"

    if [ ! -d $board ];
    then
        echo "$board is not a valid directory"
        echo "Skipping $board"
        break
    else
        cp -r .kibot $board/
    fi

    cd $board
    echo "Moved into $board directory"

    pcb_file=$(find ./kicad_* -maxdepth 1 -type f -name "*.kicad_pcb")
    if [ -z "$pcb_file" ];
    then
        echo "Failed to find KiCad 6 PCB file for $board"
        echo "Skipping $board"
        break
    else
        echo "Found PCB file $pcb_file"
    fi

    if [ -d docs ];
    then
        echo "Cleaning up old documentation"
        rm -r docs
    fi

    kibot -b $pcb_file -c ./.kibot/doc_gen.kibot.yaml
    rm -r .kibot

    mkdir ./docs/reports
    mv *-erc.txt ./docs/reports/
    mv *-drc.txt ./docs/reports/

    cd ../
    echo "Left $board directory"
done
