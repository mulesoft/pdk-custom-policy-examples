#!/bin/bash

# test-playground.sh - Test script for the block policy playground

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Source the common library
source "$PROJECT_ROOT/test-playground-common.sh"

# Setup
setup_cleanup_trap
print_header "Block Policy Playground Test"

# Verify we're in the right directory
check_directory

# Start playground and wait for initialization
start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with blocked IP (should be rejected)
print_section "Test 1: Blocked IP (24.152.57.0)"
execute_curl "Test 1" -H "ip: 24.152.57.0" "$GATEWAY_URL"
validate_status "Test 1: Blocked IP" "403" "401" "400"
# Blocked requests should not reach backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ "$LAST_HTTP_CODE" = "403" ]; then
    echo -e "${GREEN}✓ Test 1: IP properly blocked${NC}"
fi

# Test 2: Request with allowed IP (should succeed)
print_section "Test 2: Allowed IP (25.152.57.0)"
execute_curl "Test 2" -H "ip: 25.152.57.0" "$GATEWAY_URL"
validate_status "Test 2: Allowed IP" "200"
# Should get response from backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 2: Request reached backend${NC}"
fi

# Print summary and exit
print_summary "Block Policy"
exit $?
