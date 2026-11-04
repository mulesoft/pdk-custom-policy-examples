#!/bin/bash

# test-playground.sh - Test script for the ai-basic-token-rate-limiting policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "AI Basic Token Rate Limiting Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1-2: First requests should succeed
print_section "Test 1-2: Requests within token limit"
for i in {1..2}; do
    execute_curl "Test $i" -X POST "$GATEWAY_URL" \
        -H "Content-Type: application/json" \
        -d '{"model": "Llama", "messages": [{"role": "user", "content": "Give me an example of planet"}]}'
    validate_status "Test $i: Within limit" "200"
    sleep 0.5
done

# Test 3: Third request should hit token rate limit
print_section "Test 3: Request exceeding token limit"
execute_curl "Test 3" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"model": "Llama", "messages": [{"role": "user", "content": "Give me an example of planet"}]}'
validate_status "Test 3: Token limit exceeded" "403"
# Should be blocked due to token rate limit
if [ "$LAST_HTTP_CODE" = "403" ]; then
    echo -e "${GREEN}✓ Test 3: Token rate limit enforced${NC}"
fi

print_summary "AI Basic Token Rate Limiting Policy"
exit $?
