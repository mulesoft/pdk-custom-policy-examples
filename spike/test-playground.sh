#!/bin/bash

# test-playground.sh - Test script for the spike policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Spike Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Multiple requests to test spike behavior
print_section "Test 1: Request to test spike control"
execute_curl "Test 1" "$GATEWAY_URL"
validate_status "Test 1: Spike request" "200"

# Test 2: Verify spike control logs show retry information
print_section "Test 2: Verify spike control logs"
echo "Checking playground logs for retry information..."
sleep 1  # Give policy time to log

if grep -q "Retries: 0" "$PLAYGROUND_LOG"; then
    echo -e "${GREEN}✓ Test 2: Found 'Retries: 0' in logs${NC}"
    record_test_result "Test 2: Retry logging" "PASS"

    # Show the retry log line for verification
    echo "Log entry:"
    grep "Retries: 0" "$PLAYGROUND_LOG" | tail -1
else
    echo -e "${YELLOW}⚠ Test 2: 'Retries: 0' not found in logs${NC}"
    echo "Last 30 lines of playground log:"
    tail -30 "$PLAYGROUND_LOG"
    record_test_result "Test 2: Retry logging" "FAIL"
fi

print_summary "Spike Policy"
exit $?
