#!/bin/bash

# test-playground.sh - Test script for the jwt-validation policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "JWT Validation Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi


# Test 1: Request with valid JWT token should succeed
print_section "Test 1: Valid JWT token"
execute_curl "Test 1" -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMiwiZXhwIjozNTE2MjM5MDIyLCJyb2xlIjoiTWVtYmVyIn0.jN2sFykaXvVCREx3uubvHSm61yUB2NAJAxyoFB7aqQo" "$GATEWAY_URL"
validate_status "Test 1: Valid JWT" "200"

# Test 2: Request without token should be rejected
print_section "Test 2: No JWT token"
execute_curl "Test 2" "$GATEWAY_URL"
validate_status "Test 2: No token" "401" "403"

print_summary "JWT Validation Policy"
exit $?
