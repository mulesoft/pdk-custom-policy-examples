#!/bin/bash

# test-playground.sh - Test script for the crypto policy playground

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
source "$PROJECT_ROOT/test-playground-common.sh"

setup_cleanup_trap
print_header "Crypto Policy Playground Test"
check_directory

start_playground
if ! wait_for_initialization; then
    exit 1
fi

# Test 1: Request with nonce header for encryption
print_section "Test 1: Cryptographic operation with nonce"
NONCE="5c6cd51364cd25a4a25853e085ad682c899861e24f9b36934ad07503b2e70ff3fe77c8c8e38d1e00ba501d31d920b306546de0f297fbebcaffd99c4b2457f0c3996269b2ac17aec6f5c5810748f1be7a0dc20988f0a01ca61da5563e4a3f9291ba94c75912c6fa73395fd4eae3a46021e8b34bd3223b0c14d951eead7372028f04c9b7373eaee979e0bc7eaa2ad7a09cdf54c91febdb1dc0eabe35e2bc02c02a09124d45d51ed62f126e09e12dd739ed86ff578eec13b4d396c75c3bbec8a81ac3a63bfb7f20dca311bcc4fc848597dea75f0f9bc49ca6b6e6c3f1147d119b529599235045d2b2c80aa426f81494e0fe0058fd24a1766931dc07867f887c0424"
execute_curl "Test 1" -H "nonce:$NONCE" "$GATEWAY_URL"
validate_status "Test 1: With nonce" "200"

print_summary "Crypto Policy"
exit $?
