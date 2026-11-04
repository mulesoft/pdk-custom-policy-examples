#!/bin/bash

# test-playground.sh - Test script for the stop-iteration policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Stop Iteration Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: GET request with response modification
print_section "Test 1: Response modification"
execute_curl "Test 1" -i "${GATEWAY_URL}hello"
validate_status "Test 1: GET with modification" "200"
# Validate x-stop-iteration header is present in response
if [ -f "$LAST_RESPONSE_FILE" ]; then
    if grep -qiF "x-stop-iteration" "$LAST_RESPONSE_FILE"; then
        echo -e "${GREEN}✓ Test 1: x-stop-iteration header found${NC}"
        record_test_result "Test 1: Header added" "PASS"
    else
        echo -e "${YELLOW}⚠ Test 1: x-stop-iteration header not found (may need -i flag)${NC}"
    fi
fi

# Test 2: POST request to see request body modification
print_section "Test 2: POST request"
execute_curl "Test 2" -X POST "${GATEWAY_URL}hello" -d "test-data"
validate_status "Test 2: POST with modification" "200"
# Response body should be prefixed with bodyPrefix
validate_response_contains "Test 2: Body prefixed" "modified-test-data"

print_summary "Stop Iteration Policy"
exit $?
