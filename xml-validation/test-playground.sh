#!/bin/bash

# test-playground.sh - Test script for the xml-validation policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "XML Validation Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: POST with valid XML should succeed
print_section "Test 1: Valid XML"
execute_curl "Test 1" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/xml" \
    -d '<note><to>you</to><body>hello</body></note>'
validate_status "Test 1: Valid XML" "200"
# Valid XML should pass through to backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 1: Valid XML passed validation${NC}"
fi

# Test 2: POST with malformed XML should return 400
print_section "Test 2: Malformed XML"
execute_curl "Test 2" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/xml" \
    -d '<note><to></note>'
validate_status "Test 2: Malformed XML" "400"
# Should return error response about malformed XML
if [ -f "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 2: Malformed XML rejected${NC}"
fi

# Test 3: GET without body should succeed (validation skipped)
print_section "Test 3: GET without body"
execute_curl "Test 3" "$GATEWAY_URL"
validate_status "Test 3: No body" "200"
# Should pass through when no body present
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 3: Request without body passed${NC}"
fi

print_summary "XML Validation Policy"
exit $?
