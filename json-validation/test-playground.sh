#!/bin/bash

# test-playground.sh - Test script for the json-validation policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "JSON Validation Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: POST with valid JSON should succeed
print_section "Test 1: Valid JSON"
execute_curl "Test 1" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"item":"book","qty":1}'
validate_status "Test 1: Valid JSON" "200"
# Valid JSON should pass through to backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 1: Valid JSON passed validation${NC}"
fi

# Test 2: POST with invalid JSON should return 400
print_section "Test 2: Invalid JSON"
execute_curl "Test 2" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"a":1]'
validate_status "Test 2: Invalid JSON" "400"
# Should return error response about invalid JSON
if [ -f "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 2: Invalid JSON rejected${NC}"
fi

# Test 3: GET without body should succeed as validation is skipped
print_section "Test 3: GET without body"
execute_curl "Test 3" "$GATEWAY_URL"
validate_status "Test 3: No body" "200"
# Should pass through when no body present
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 3: Request without body passed${NC}"
fi

print_summary "JSON Validation Policy"
exit $?
