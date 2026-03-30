#!/bin/bash

root=$(pwd)

for f in $(find . | grep '.*Cargo.toml'); do
    f=$(dirname "$f")
    if [ -d "$f" ]; then
        echo "Running unit tests for '$f'.";
        cd "$f";
        RUST_BACKTRACE=1 cargo test --lib || exit 1;
        cd "$root";
    fi
done
