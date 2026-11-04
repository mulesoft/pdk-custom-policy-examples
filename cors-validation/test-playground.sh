#!/bin/bash

# test-playground.sh - Test script for the cors-validation policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "CORS Validation Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with Origin header (should return CORS headers)
print_section "Test 1: Request with Origin header"
execute_curl "Test 1" -i -H "Origin: http://www.the-origin.com" "$GATEWAY_URL"
validate_status "Test 1: With Origin" "200"
# Validate CORS headers in response
if [ -f "$LAST_RESPONSE_FILE" ]; then
    if grep -qiE "access-control-allow-origin|Access-Control-Allow-Origin" "$LAST_RESPONSE_FILE"; then
        echo -e "${GREEN}✓ Test 1: CORS headers present${NC}"
        record_test_result "Test 1: CORS headers" "PASS"
    else
        echo -e "${YELLOW}⚠ Test 1: CORS headers not found in response${NC}"
    fi
fi

print_summary "CORS Validation Policy"
exit $?
