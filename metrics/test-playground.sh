#!/bin/bash

# test-playground.sh - Test script for the metrics policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Metrics Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Basic connectivity test
print_section "Test 1: Basic request"
execute_curl "Test 1" "$GATEWAY_URL"
validate_status "Test 1: Basic request" "200"

# Test 2: Verify metrics were posted in logs
print_section "Test 2: Verify metrics posting"
echo "Checking playground logs for metrics posting..."
sleep 2  # Give policy time to post metrics

if grep -q "Metrics posted successfully!" "$PLAYGROUND_LOG"; then
    echo -e "${GREEN}✓ Test 2: Metrics posted successfully log found${NC}"
    record_test_result "Test 2: Metrics posted" "PASS"
else
    echo -e "${RED}✗ Test 2: Metrics posted log not found${NC}"
    echo "Last 20 lines of playground log:"
    tail -20 "$PLAYGROUND_LOG"
    record_test_result "Test 2: Metrics posted" "FAIL"
fi

print_summary "Metrics Policy"
exit $?
