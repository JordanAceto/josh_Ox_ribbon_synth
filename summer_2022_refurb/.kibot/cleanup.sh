#!/bin/bash

boards=("$@")

for board in "${boards[@]}"
do
    echo "Deleting documentation for $board"
    cd $board

    if [ -d docs ];
    then
        rm -r docs
        rm -f index.html
        rm -f *screencast.ogv
    else
        echo "No docs found for $board"
    fi

    cd ../
done
