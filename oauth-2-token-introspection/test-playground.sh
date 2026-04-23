#!/bin/bash

# test-playground.sh - Test script for the oauth-2-token-introspection policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "OAuth 2.0 Token Introspection Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with valid OAuth2 token (mock server always returns valid)
print_section "Test 1: Valid OAuth2 token"
execute_curl "Test 1" -H "Authorization: Bearer valid-token-12345" "$GATEWAY_URL"
validate_status "Test 1: Valid token" "200"
# With a valid token, request should reach backend
if [ -f "$LAST_RESPONSE_FILE" ] && [ -s "$LAST_RESPONSE_FILE" ]; then
    echo -e "${GREEN}✓ Test 1: Valid token allowed, request reached backend${NC}"
    # Check if response contains backend echo data
    if grep -q '"headers"' "$LAST_RESPONSE_FILE"; then
        echo -e "${GREEN}✓ Test 1: Backend response received${NC}"
        record_test_result "Test 1: Backend response" "PASS"
    fi
fi

# Test 2: Request without token (should be rejected)
print_section "Test 2: No OAuth2 token"
execute_curl "Test 2" "$GATEWAY_URL"
validate_status "Test 2: No token" "400"
# Without token, should be rejected before reaching backend
if [ "$LAST_HTTP_CODE" = "400" ]; then
    echo -e "${GREEN}✓ Test 2: Request without token properly rejected${NC}"
fi

# Test 3: Request with malformed Authorization header
print_section "Test 3: Malformed Authorization header"
execute_curl "Test 3" -H "Authorization: InvalidFormat" "$GATEWAY_URL"
validate_status "Test 3: Malformed header" "401" "403" "400"
# Malformed authorization should be rejected
if [[ "$LAST_HTTP_CODE" == "401" || "$LAST_HTTP_CODE" == "403" || "$LAST_HTTP_CODE" == "400" ]]; then
    echo -e "${GREEN}✓ Test 3: Malformed authorization properly rejected${NC}"
fi

# Test 4: Request with token but missing Bearer prefix
print_section "Test 4: Token without Bearer prefix"
execute_curl "Test 4" -H "Authorization: just-a-token" "$GATEWAY_URL"
validate_status "Test 4: No Bearer prefix" "401" "403" "400"
# Should be rejected due to improper format
if [[ "$LAST_HTTP_CODE" == "401" || "$LAST_HTTP_CODE" == "403" || "$LAST_HTTP_CODE" == "400" ]]; then
    echo -e "${GREEN}✓ Test 4: Invalid format properly rejected${NC}"
fi

# Test 5: Verify introspection logs (optional - check if introspection occurred)
print_section "Test 5: Verify token introspection occurred"
echo "Checking playground logs for introspection activity..."
if [ -f "$PLAYGROUND_LOG" ]; then
    if grep -qi "introspection\|oauth\|token" "$PLAYGROUND_LOG"; then
        echo -e "${GREEN}✓ Test 5: Token introspection activity found in logs${NC}"
        record_test_result "Test 5: Introspection logs" "PASS"
    else
        echo -e "${YELLOW}⚠ Test 5: No explicit introspection logs found (this may be normal)${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Test 5: Playground log not accessible${NC}"
fi

print_summary "OAuth 2.0 Token Introspection Policy"
exit $?
