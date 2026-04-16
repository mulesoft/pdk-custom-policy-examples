#!/bin/bash

# test-playground.sh - Test script for the query policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Query Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with query parameters
print_section "Test 1: Query parameter handling"
execute_curl "Test 1" "${GATEWAY_URL}?key=value&extra&absent=absent"
validate_status "Test 1: Query parameters" "200"
validate_response_contains "Test 1: X-Query-Extra header" '"X-Query-Extra"'
validate_response_contains "Test 1: X-Query-Key header" '"X-Query-Key"'
validate_response_contains "Test 1: Removed params" '"removed"'

print_summary "Query Policy"
exit $?
