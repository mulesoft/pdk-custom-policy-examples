#!/bin/bash

# test-playground.sh - Test script for the simple-oauth-2-validation-grpc policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Simple OAuth 2 Validation gRPC Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with valid token
print_section "Test 1: Valid token"
execute_curl "Test 1" "${GATEWAY_URL}hello?token=valid"
validate_status "Test 1: Valid token" "200"

# Test 2: Request with invalid token
print_section "Test 2: Invalid token"
execute_curl "Test 2" "${GATEWAY_URL}hello?token=not_valid"
validate_status "Test 2: Invalid token" "401" "403"

print_summary "Simple OAuth 2 Validation gRPC Policy"
exit $?
