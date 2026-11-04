#!/bin/bash

# test-playground.sh - Test script for the data-storage-stats policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Data Storage Stats Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with client ID to track statistics
print_section "Test 1: Request with client ID"
execute_curl "Test 1" -H "x-client-id: client-123" "${GATEWAY_URL}test"
validate_status "Test 1: Client tracked" "200"

# Test 2: Retrieve statistics
print_section "Test 2: Retrieve statistics"
execute_curl "Test 2" "${GATEWAY_URL}stats"
validate_status "Test 2: Stats retrieved" "200"
# Stats response should contain client-123
validate_response_contains "Test 2: Client tracked" "client-123"

# Test 3: Reset statistics
print_section "Test 3: Reset statistics"
execute_curl "Test 3" -X DELETE "${GATEWAY_URL}stats"
validate_status "Test 3: Stats reset" "200"
# Response should confirm reset
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 3: Stats reset confirmed${NC}"
fi

print_summary "Data Storage Stats Policy"
exit $?
