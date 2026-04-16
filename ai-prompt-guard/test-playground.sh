#!/bin/bash

# test-playground.sh - Test script for the ai-prompt-guard policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "AI Prompt Guard Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: POST with email (should delete email token and succeed)
print_section "Test 1: Email in prompt (should be guarded)"
execute_curl "Test 1" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"model": "llama", "messages": [{"role": "user", "content": "My email es flexmaster@salesforce.com"}]}'
validate_status "Test 1: Email guarded" "200"
# Email should be removed/masked in the response
echo -e "${YELLOW}Note: Response should have email removed or masked${NC}"

# Test 2: POST with email and phone (should return 403)
print_section "Test 2: Email and phone in prompt (should be blocked)"
execute_curl "Test 2" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"model": "llama", "messages": [{"role": "user", "content": "My email es flexmaster@salesforce.com and my phone number is +1 9343 6126649"}]}'
validate_status "Test 2: Multiple PII blocked" "403"

print_summary "AI Prompt Guard Policy"
exit $?
