#!/bin/bash

#./.scripts/update-dep.sh pdk "https://github.com/mulesoft-emu/mgw-policy-development-kit.git" "7f561a6148de8ff81cd9ef936ddec5e97fb76b26"

print_usage() {
    echo "Usage: ./update-dep.sh <dependency> <version>";
    echo "Usage: ./update-dep.sh <dependency> <version> --dev";
    echo "Usage: ./update-dep.sh <dependency> <git-repo> <git-rev>";
    echo "Usage: ./update-dep.sh <dependency> <git-repo> <git-rev> --dev";
}

if [ -z "$1" ]; then
    print_usage;
    exit 1;
fi

if [ -z "$2" ]; then
    echo "Error: <version> or <git-repo> is required.";
    print_usage;
    exit 2;
fi

if [[ "$2" == http* ]] && [ -z "$3" ]; then
    echo "Error: <git-rev> is required when <git-repo> is provided.";
    print_usage;
    exit 3;
fi

if [ -n "$4" ] && [ "$4" != "--dev" ]; then
    echo "Error: unexpected argument '$4'. Only '--dev' is allowed as the 4th parameter.";
    print_usage;
    exit 4;
fi

if [[ "$2" == http* ]]; then
    dev_flag="${4}";
else
    dev_flag="${3}";
fi

root=$(pwd)

for f in $(find . | grep '.*Cargo.toml'); do
    f=$(dirname "$f")
    if [ -d "$f" ] && [[ "$f" != *"agent-policies"* ]]; then
        echo "Updating dependency '$1' in '$f'.";
        cd "$f"
        if [[ "$2" == http* ]]; then
            cargo add "$1" --git $2 --rev $3 $dev_flag
        else
            cargo add "$1@$2" $dev_flag
        fi

        cd "$root";
    fi
done
