#!/bin/bash

# test-playground.sh - Test script for the certs policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Certs Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: HTTPS request with client certificates
print_section "Test 1: Certificate extraction"

execute_curl "Test 1" "https://localhost:8081" \
  --cert tests/resources/client.pem \
  --key tests/resources/client.key \
  --cacert tests/resources/ca.pem \
  --tls-max 1.2 \
  -v
validate_status "Test 1: With certificates" "200"
validate_response_contains "Test 1: Certificate added" '"X-Peer-Email": "joker@phantomthieves.com"'


print_summary "Certs Policy"
exit $?
