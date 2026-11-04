#!/bin/bash

# test-playground.sh - Test script for the spike-control policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Spike Control Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1-2: Send requests within limit (should succeed)
print_section "Test 1-10: Requests within spike limit"
for i in {1..2}; do
    execute_curl "Test $i" "$GATEWAY_URL"
    validate_status "Test $i: Within limit" "200"
    sleep 0.1
done

# Test 3: Request exceeding limit (should be rate limited)
print_section "Test 3: Request exceeding spike limit"
execute_curl "Test 3" "$GATEWAY_URL"
validate_status "Test 3: Exceeding limit" "429"
# Rate limited response should not contain backend data
if [ "$LAST_HTTP_CODE" = "429" ]; then
    echo -e "${GREEN}✓ Test 3: Request properly rate limited${NC}"
fi

print_summary "Spike Control Policy"
exit $?
