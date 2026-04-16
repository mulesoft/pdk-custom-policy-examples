#!/bin/bash

# test-playground.sh - Test script for the data-caching policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Data Caching Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with cache header and path
print_section "Test 1: First request with cache header"
execute_curl "Test 1" -H "cache_check: cache_value" "${GATEWAY_URL}catalog/1"
validate_status "Test 1: Cached request" "200"
validate_response_contains "Test 1: Cache value in response" '"Cache-Check": "cache_value"'

# Test 2: Request with different cache header value
print_section "Test 2: Second request with different cache header"
execute_curl "Test 2" -H "cache_check: cache_value1" "${GATEWAY_URL}catalog/1"
validate_status "Test 2: Different cache value" "200"
validate_response_contains "Test 2: Cached value returned" '"Cache-Check": "cache_value"'

print_summary "Data Caching Policy"
exit $?
