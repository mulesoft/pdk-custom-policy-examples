#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: ./update-dep.sh <dependency> <version>";
    echo "Usage: ./update-dep.sh <dependency> <git-repo> <git-rev>";
    exit 1;
fi

if [ -z "$2" ]; then
    echo "No version to update was given";
    exit 2;
fi

root=$(pwd)

for f in $(find . | grep '.*Cargo.toml'); do
    f=$(dirname "$f")
    if [ -d "$f" ]; then
        echo "Updating dependency '$1' in '$f'.";
        cd "$f"
        if [ -z "$3" ]; then
            cargo add "$1@$2"
        else
            cargo add "$1" --git $2 --rev $3
        fi

        cd "$root";
    fi
done
