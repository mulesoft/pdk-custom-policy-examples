#!/bin/bash
# Copyright 2023 Salesforce, Inc. All rights reserved.
#
# Single entry point to run the integration tests of every policy example.
#
# Usage:
#   ./.scripts/test.sh <path-to-registration.yaml>
#
# Environment:
#   PDK_TEST_FLEX_IMAGE_VERSION  Flex version under test (e.g. 1.9.3000). Used to
#                                skip examples that declare a higher minimum Flex
#                                version. Also consumed by pdk-test to select the
#                                Flex image. When unset or "latest", no example is
#                                skipped.
#
# An example declares its minimum Flex version in Cargo.toml:
#
#   [package.metadata.flex]
#   min-version = "1.12.0"
#
# Examples requiring a version newer than PDK_TEST_FLEX_IMAGE_VERSION are skipped,
# so any job running these tests inherits the same compatibility rules. Examples
# without a declaration have no minimum and always run.

if [ -z "$1" ]; then
    echo "No path to registration file was given";
    echo "Usage: $0 <path-to-registration.yaml>";
    exit 1;
fi

REGISTRATION_FILE="$1"

if [ ! -f "$REGISTRATION_FILE" ]; then
    echo "Error: registration file not found: $REGISTRATION_FILE";
    exit 1;
fi

FLEX_VERSION="${PDK_TEST_FLEX_IMAGE_VERSION:-}"

# Returns 0 (true) when $1 < $2 numerically, comparing dot-separated components.
# Missing components default to 0, so "1.12" == "1.12.0".
version_lt() {
    local v1="$1" v2="$2" i
    local -a p1 p2
    IFS='.' read -r -a p1 <<< "$v1"
    IFS='.' read -r -a p2 <<< "$v2"
    local max="${#p1[@]}"
    [ "${#p2[@]}" -gt "$max" ] && max="${#p2[@]}"
    for ((i = 0; i < max; i++)); do
        local a="${p1[i]:-0}" b="${p2[i]:-0}"
        # Non-numeric component (e.g. a suffix) => treat as equal, stop comparing.
        [[ "$a" =~ ^[0-9]+$ ]] || return 1
        [[ "$b" =~ ^[0-9]+$ ]] || return 1
        if [ "$a" -lt "$b" ]; then return 0; fi
        if [ "$a" -gt "$b" ]; then return 1; fi
    done
    return 1
}

# Prints the min-version declared under [package.metadata.flex] in the
# example's Cargo.toml, or nothing when absent. Uses awk (jq is not available in
# CI) and requires no build.
min_flex_version() {
    local cargo="${1}Cargo.toml"
    [ -f "$cargo" ] || return 0
    awk '
        /^\[package\.metadata\.flex\]/ { in_section = 1; next }
        /^\[/                          { in_section = 0 }
        in_section && /^[[:space:]]*min-version[[:space:]]*=/ {
            gsub(/^[^"]*"/, ""); gsub(/".*$/, ""); print; exit
        }
    ' "$cargo"
}

# Returns 0 (true) when the example in $1 must be skipped for FLEX_VERSION.
should_skip() {
    local dir="$1"
    # Unknown / rolling versions run everything.
    [ -z "$FLEX_VERSION" ] && return 1
    [ "$FLEX_VERSION" = "latest" ] && return 1
    local min
    min="$(min_flex_version "$dir")"
    # No declaration => no minimum => always run.
    [ -z "$min" ] && return 1
    version_lt "$FLEX_VERSION" "$min"
}

echo "PDK Custom Policy Examples tests starting..."
echo "Flex version under test: ${FLEX_VERSION:-<unset>}"

total=0
success=0
failed=0
skipped=0

set +e

for dir in */; do
    if [ -f "${dir}Makefile" ] && [ -f "${dir}Cargo.toml" ]; then
        dir_name="${dir%/}"

        if should_skip "$dir"; then
            skipped=$((skipped + 1))
            echo ""
            echo "Skipping ${dir_name} (requires Flex >= $(min_flex_version "$dir"), testing ${FLEX_VERSION})"
            continue
        fi

        total=$((total + 1))

        echo ""
        echo "Testing ${dir_name}..."

        cd "$dir" || { failed=$((failed + 1)); continue; }

        # Provide the registration file wherever the example expects it.
        if [ -d "tests/config" ]; then
            cp "$REGISTRATION_FILE" tests/config/registration.yaml
        elif [ -d "tests/common" ]; then
            cp "$REGISTRATION_FILE" tests/common/registration.yaml
        fi

        make setup && make build

        accum=0
        tests=$(cargo test -- --list 2>/dev/null | grep ": test" | sed 's|\(.*\): test|\1|g')
        for single_test in $tests; do
            echo "Running test '$single_test'";
            cargo test "$single_test" -- --nocapture;
            accum=$((accum + $?))
        done

        if [ $accum -eq 0 ]; then
            success=$((success + 1))
            echo "PASSED: ${dir_name}"
        else
            failed=$((failed + 1))
            echo "FAILED: ${dir_name}"

            echo ""
            echo "==================== Flex Logs for ${dir_name} ===================="
            for log_file in $(find target/pdk-test -name "local-flex.log" 2>/dev/null); do
                echo "--- ${log_file} ---"
                cat "${log_file}" 2>/dev/null || echo "(empty or not found)"
                echo ""
            done
            echo "=================================================================="
        fi

        cd .. || exit 1
    fi
done

echo ""
echo "============================== Examples Test Summary =============================="
echo "Total: $total | Success: $success | Failed: $failed | Skipped: $skipped"

if [ $failed -gt 0 ]; then
    echo "ERROR: $failed examples failed"
    exit 1
fi
