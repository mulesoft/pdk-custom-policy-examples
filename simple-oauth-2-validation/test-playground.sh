#!/bin/bash

# test-playground.sh - Test script for the simple-oauth-2-validation policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Simple OAuth 2 Validation Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request without token (should be rejected)
print_section "Test 1: No OAuth token"
execute_curl "Test 1" "$GATEWAY_URL"
validate_status "Test 1: No token" "401" "403"

# Test 2: Request with OAuth token
print_section "Test 2: With OAuth token"
execute_curl "Test 2" -H "Authorization: Bearer test-token" "$GATEWAY_URL"
validate_status "Test 2: With token" "200"
# With valid token, request should reach backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 2: Token accepted, request reached backend${NC}"
fi

print_summary "Simple OAuth 2 Validation Policy"
exit $?
