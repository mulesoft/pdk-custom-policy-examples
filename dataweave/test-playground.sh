#!/bin/bash

# test-playground.sh - Test script for the dataweave policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Dataweave Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with basic auth should return expression result
print_section "Test 1: Dataweave expression evaluation"
execute_curl "Test 1" -u "user:pass" "$GATEWAY_URL"
validate_status "Test 1: Expression evaluated" "200"
validate_response_contains "Test 1: Expected result" '{"result":["user","pass"]}'

print_summary "Dataweave Policy"
exit $?
