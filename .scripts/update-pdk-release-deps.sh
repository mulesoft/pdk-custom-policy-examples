#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: .scripts/update-pdk-release-deps.sh <ver>";
    exit 1;
fi

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

$SCRIPT_DIR/update-dep.sh pdk "$1"
$SCRIPT_DIR/update-dep.sh pdk-test "$1" --dev
$SCRIPT_DIR/update-dep.sh pdk-unit "$1" --dev