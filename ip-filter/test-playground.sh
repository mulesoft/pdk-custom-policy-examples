#!/bin/bash

# test-playground.sh - Test script for the ip-filter policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "IP Filter Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Allowed IP from CIDR range should succeed
print_section "Test 1: Allowed IP (192.168.1.100)"
execute_curl "Test 1" -H "x-real-ip: 192.168.1.100" "${GATEWAY_URL}anything/echo/"
validate_status "Test 1: Allowed IP" "200"

# Test 2: Specific allowed IP should succeed
print_section "Test 2: Specific allowed IP (10.0.0.1)"
execute_curl "Test 2" -H "x-real-ip: 10.0.0.1" "${GATEWAY_URL}anything/echo/"
validate_status "Test 2: Specific allowed IP" "200"

# Test 3: Blocked IP should be rejected
print_section "Test 3: Blocked IP (192.168.1.50)"
execute_curl "Test 3" -H "x-real-ip: 192.168.1.50" "${GATEWAY_URL}anything/echo/"
validate_status "Test 3: Blocked IP" "403"
# Blocked IP should not reach backend
if [ "$LAST_HTTP_CODE" = "403" ]; then
    echo -e "${GREEN}✓ Test 3: IP correctly blocked${NC}"
fi

# Test 4: IP not in allowlist should be rejected
print_section "Test 4: Not in allowlist (8.8.8.8)"
execute_curl "Test 4" -H "x-real-ip: 8.8.8.8" "${GATEWAY_URL}anything/echo/"
validate_status "Test 4: Not in allowlist" "403"
# Non-allowed IP should not reach backend
if [ "$LAST_HTTP_CODE" = "403" ]; then
    echo -e "${GREEN}✓ Test 4: IP correctly rejected${NC}"
fi

print_summary "IP Filter Policy"
exit $?
