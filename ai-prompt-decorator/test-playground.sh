#!/bin/bash

# test-playground.sh - Test script for the ai-prompt-decorator policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "AI Prompt Decorator Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: POST with AI prompt
print_section "Test 1: AI prompt decoration"
execute_curl "Test 1" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"model": "llama", "messages": [{"role": "user", "content": "Give me an example of exo-planet"}]}'
validate_status "Test 1: Prompt decorated" "200"
validate_response_contains "Test 1: System role added" '{\"model\":\"llama\",\"messages\":[{\"role\":\"system\",\"content\":\"You are astronomer.\"},{\"role\":\"user\",\"content\":\"Focus on solar system.\"},{\"role\":\"user\",\"content\":\"Give me an example of exo-planet\"},{\"role\":\"user\",\"content\":\"Do not use speculative theories.\"}]}'


print_summary "AI Prompt Decorator Policy"
exit $?
