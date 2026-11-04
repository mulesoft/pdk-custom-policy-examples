#!/bin/bash

root=$(pwd)

for f in $(find . | grep '.*Cargo.toml'); do
    f=$(dirname "$f")
    if [ -d "$f" ]; then
        echo "Formatting '$f'.";
        cd "$f";
        cargo fmt;
        cd "$root";
    fi
done
