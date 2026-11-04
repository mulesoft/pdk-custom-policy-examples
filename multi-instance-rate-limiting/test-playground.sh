#!/bin/bash

# test-playground.sh - Test script for the multi-instance-rate-limiting policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Multi Instance Rate Limiting Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1-3: Requests with API key within limit
print_section "Test 1-3: Requests with x-api-key header"
for i in {1..3}; do
    execute_curl "Test $i" -H "x-api-key: key-1" "${GATEWAY_URL}anything/echo/"
    validate_status "Test $i: With API key" "200"
    sleep 0.2
done

execute_curl "Test $i" -H "x-api-key: key-1" "${GATEWAY_URL}anything/echo/"
validate_status "Test 4: With API key" "429"

print_summary "Multi Instance Rate Limiting Policy"
exit $?
