#!/bin/bash

root=$(pwd)

for f in $(find . | grep '.*Cargo.toml'); do
    f=$(dirname "$f")
    if [ -d "$f" ]; then
        echo "Building '$f' policy example";
        cd "$f";
        make setup;
        make build;
        cd "$root";
    fi
done
