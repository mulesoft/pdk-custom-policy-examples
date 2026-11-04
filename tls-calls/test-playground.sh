#!/bin/bash

# test-playground.sh - Test script for the tls-calls policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "tls calls Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Basic connectivity test
print_section "Test 1: Basic request"
execute_curl "Test 1" "$GATEWAY_URL"
validate_status "Test 1: Basic request" "200"

print_summary "tls calls Policy"
exit $?
