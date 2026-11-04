#!/bin/bash

if [ -z "$1" ]; then
    echo "Usage: .scripts/update-pdk-deps.sh <git-rev>";
    exit 1;
fi

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

$SCRIPT_DIR/update-dep.sh pdk "https://github.com/mulesoft-emu/mgw-policy-development-kit.git" "$1"
$SCRIPT_DIR/update-dep.sh pdk-test "https://github.com/mulesoft-emu/mgw-policy-development-kit.git" "$1" --dev
$SCRIPT_DIR/update-dep.sh pdk-unit "https://github.com/mulesoft-emu/mgw-policy-development-kit.git" "$1" --dev
