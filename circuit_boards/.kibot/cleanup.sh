#!/bin/bash

boards=("$@")

cd ../

for board in "${boards[@]}"
do
    echo "Deleting documentation for $board"
    cd $board

    [ ! -d docs ] || rm -r docs
    [ ! -d .kibot ] || rm -r .kibot
    [ ! -e index.html ] || rm -f index.html
    [ ! -e *-drc.txt ] || rm -f *-drc.txt
    [ ! -e *-erc.txt ] || rm -f *-erc.txt

    cd ../
done
