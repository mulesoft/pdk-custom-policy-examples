#!/bin/bash

# test-playground.sh - Test script for the jwt-generation policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "JWT Generation Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request should return JWT in response body
print_section "Test 1: JWT Generation"
execute_curl "Test 1" "$GATEWAY_URL"
validate_status "Test 1: JWT in response" "200"
# Validate JWT format (header.payload.signature)
if [ -f "$LAST_RESPONSE_FILE" ]; then
    if grep -qE '^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+' "$LAST_RESPONSE_FILE"; then
        echo -e "${GREEN}✓ Test 1: JWT format validated${NC}"
        record_test_result "Test 1: JWT format" "PASS"
    else
        echo -e "${RED}✗ Test 1: Response doesn't contain valid JWT format${NC}"
        record_test_result "Test 1: JWT format" "FAIL"
    fi
fi

print_summary "JWT Generation Policy"
exit $?
