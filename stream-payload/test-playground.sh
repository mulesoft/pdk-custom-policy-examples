#!/bin/bash

# test-playground.sh - Test script for the stream-payload policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Stream Payload Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with forbidden string in payload
print_section "Test 1: Forbidden string detection"
execute_curl "Test 1" -d '${jndi' "$GATEWAY_URL"
validate_status "Test 1: Forbidden string blocked" "400" "403"
# Should block forbidden string and return error
if [[ "$LAST_HTTP_CODE" == "400" || "$LAST_HTTP_CODE" == "403" ]]; then
    echo -e "${GREEN}✓ Test 1: Forbidden string properly blocked${NC}"
fi

# Test 2: Normal request without forbidden string
print_section "Test 2: Normal request"
execute_curl "Test 2" "$GATEWAY_URL"
validate_status "Test 2: Normal request" "200"
# Normal request should reach backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 2: Normal request passed through${NC}"
fi

print_summary "Stream Payload Policy"
exit $?
