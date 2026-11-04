#!/bin/bash

# test-playground.sh - Test script for the ai-prompt-template policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "AI Prompt Template Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: POST with template properties
print_section "Test 1: Template application"
execute_curl "Test 1" -X POST "$GATEWAY_URL" \
    -H "Content-Type: application/json" \
    -d '{"prompt": "{template://veterinarian-chat}", "properties": {"species": "falcon", "system": "respiratory"}}'
validate_status "Test 1: Template applied" "200"
validate_response_contains "Test 1: System role added" '"You are a respiratory expert, in falcon species."'
validate_response_contains "Test 1: User role added" '"You are a respiratory expert, in falcon species."'

print_summary "AI Prompt Template Policy"
exit $?
