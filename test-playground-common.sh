#!/bin/bash

# test-playground-common.sh - Common library for playground testing scripts
# This library provides reusable functions for testing PDK policy playgrounds

# Colors for output
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export NC='\033[0m' # No Color

# Global variables
export PLAYGROUND_LOG="/tmp/playground-$$.log"
export PLAYGROUND_PID=""
export TIMEOUT=120
export GATEWAY_URL="http://127.0.0.1:8081/"

# Test results tracking (bash 3.2 compatible - using parallel arrays)
export TEST_RESULTS_NAMES=""
export TEST_RESULTS_STATUS=""
export TEST_COUNT=0

# Helper function to record test result
record_test_result() {
    local test_name="$1"
    local status="$2"

    if [ -z "$TEST_RESULTS_NAMES" ]; then
        TEST_RESULTS_NAMES="$test_name"
        TEST_RESULTS_STATUS="$status"
    else
        TEST_RESULTS_NAMES="${TEST_RESULTS_NAMES}|${test_name}"
        TEST_RESULTS_STATUS="${TEST_RESULTS_STATUS}|${status}"
    fi
}

# Cleanup function
cleanup() {
    if [ -n "$PLAYGROUND_PID" ] && kill -0 "$PLAYGROUND_PID" 2>/dev/null; then
        kill "$PLAYGROUND_PID" 2>/dev/null || true
        wait "$PLAYGROUND_PID" 2>/dev/null || true
    fi
    if [ -f "$PLAYGROUND_LOG" ]; then
        rm -f "$PLAYGROUND_LOG"
    fi
    # Clean up any temp response files
    rm -f /tmp/response*-$$.txt 2>/dev/null || true
}

# Setup trap for cleanup
setup_cleanup_trap() {
    trap cleanup EXIT INT TERM
}

# Print header
print_header() {
    local title="$1"
    echo "=========================================="
    echo "$title"
    echo "=========================================="
    echo ""
}

# Print section
print_section() {
    local title="$1"
    echo ""
    echo "=========================================="
    echo "$title"
    echo "=========================================="
    echo ""
}

# Check if we're in the right directory
check_directory() {
    if [ ! -f "Makefile" ] || [ ! -d "playground" ]; then
        echo -e "${RED}Error: This script must be run from a policy directory with a playground folder${NC}"
        exit 1
    fi
}

# Start the playground
start_playground() {
    echo -e "${YELLOW}Starting playground with 'make run'...${NC}"
    make run > "$PLAYGROUND_LOG" 2>&1 &
    PLAYGROUND_PID=$!
    echo "Playground started (PID: $PLAYGROUND_PID)"
    echo ""
}

# Wait for playground initialization
wait_for_initialization() {
    echo -e "${YELLOW}Waiting for playground to initialize...${NC}"
    local INIT_MSG_FOUND=false
    local LDS_MSG_FOUND=false
    local START_TIME=$(date +%s)

    while true; do
        local CURRENT_TIME=$(date +%s)
        local ELAPSED=$((CURRENT_TIME - START_TIME))

        if [ $ELAPSED -gt $TIMEOUT ]; then
            echo -e "${RED}Timeout: Playground did not initialize within ${TIMEOUT} seconds${NC}"
            echo ""
            echo "Last 50 lines of playground log:"
            tail -50 "$PLAYGROUND_LOG"
            return 1
        fi

        # Check if playground process is still running
        if ! kill -0 "$PLAYGROUND_PID" 2>/dev/null; then
            echo -e "${RED}Error: Playground process died unexpectedly${NC}"
            echo ""
            echo "Playground log:"
            cat "$PLAYGROUND_LOG"
            return 1
        fi

        # Check for initialization message
        if ! $INIT_MSG_FOUND && grep -q "all dependencies initialized. starting workers" "$PLAYGROUND_LOG"; then
            echo -e "${GREEN}✓${NC} Found: 'all dependencies initialized. starting workers'"
            INIT_MSG_FOUND=true
        fi

        # Check for LDS listener message (must come AFTER init message in the file)
        if $INIT_MSG_FOUND && ! $LDS_MSG_FOUND && grep -q "lds: add/update listener 'listener-8081'" "$PLAYGROUND_LOG"; then
            # Verify the ordering: get line numbers of both messages
            # Use head -1 for init (happens once) and tail -1 for LDS (may appear multiple times)
            local INIT_LINE=$(grep -n "all dependencies initialized. starting workers" "$PLAYGROUND_LOG" | head -1 | cut -d: -f1)
            local LDS_LINE=$(grep -n "lds: add/update listener 'listener-8081'" "$PLAYGROUND_LOG" | tail -1 | cut -d: -f1)

            # Ensure LDS message comes AFTER init message (checking LAST occurrence of LDS)
            if [ "$LDS_LINE" -gt "$INIT_LINE" ]; then
                echo -e "${GREEN}✓${NC} Found: 'lds: add/update listener 'listener-8081'' (after init message)"
                LDS_MSG_FOUND=true
                break
            else
                echo -e "${YELLOW}⚠${NC} Found LDS message but latest occurrence is before init message, waiting..."
            fi
        fi

        sleep 1
    done

    echo -e "${GREEN}Playground initialized successfully!${NC}"
    echo ""

    # Give it a moment to be fully ready
    sleep 2
    return 0
}

# Execute a curl command and capture response
# Usage: execute_curl <test_name> <curl_args...>
# Returns: Sets LAST_HTTP_CODE and LAST_RESPONSE_FILE
execute_curl() {
    local test_name="$1"
    shift
    local curl_args=("$@")

    TEST_COUNT=$((TEST_COUNT + 1))
    local response_file="/tmp/response${TEST_COUNT}-$$.txt"

    echo "Executing: curl ${curl_args[*]}"
    echo ""

    LAST_HTTP_CODE=$(curl -s -o "$response_file" -w "%{http_code}" "${curl_args[@]}")
    LAST_RESPONSE_FILE="$response_file"

    return 0
}

# Validate HTTP status code
# Usage: validate_status <test_name> <expected_code> [<alternate_code>...]
validate_status() {
    local test_name="$1"
    shift
    local expected_codes=("$@")

    for code in "${expected_codes[@]}"; do
        if [ "$LAST_HTTP_CODE" = "$code" ]; then
            echo -e "${GREEN}✓ $test_name PASSED: HTTP $LAST_HTTP_CODE${NC}"
            record_test_result "$test_name" "PASS"
            return 0
        fi
    done

    echo -e "${RED}✗ $test_name FAILED: Expected HTTP ${expected_codes[*]}, got HTTP $LAST_HTTP_CODE${NC}"
    if [ -f "$LAST_RESPONSE_FILE" ]; then
        echo "Response:"
        cat "$LAST_RESPONSE_FILE"
    fi
    record_test_result "$test_name" "FAIL"
    return 1
}

# Validate response contains text
# Usage: validate_response_contains <test_name> <expected_text>
validate_response_contains() {
    local test_name="$1"
    local expected_text="$2"

    if [ ! -f "$LAST_RESPONSE_FILE" ]; then
        echo -e "${RED}✗ $test_name FAILED: No response file found${NC}"
        record_test_result "$test_name" "FAIL"
        return 1
    fi

    if grep -qF "$expected_text" "$LAST_RESPONSE_FILE"; then
        echo -e "${GREEN}✓ $test_name PASSED: Response contains '$expected_text'${NC}"
        record_test_result "$test_name" "PASS"
        return 0
    else
        echo -e "${RED}✗ $test_name FAILED: Response does not contain '$expected_text'${NC}"
        echo "Response:"
        cat "$LAST_RESPONSE_FILE"
        record_test_result "$test_name" "FAIL"
        return 1
    fi
}

# Validate response header
# Usage: validate_header <test_name> <header_name> <expected_value>
validate_header() {
    local test_name="$1"
    local header_name="$2"
    local expected_value="$3"

    TEST_COUNT=$((TEST_COUNT + 1))
    local response_file="/tmp/response${TEST_COUNT}-$$.txt"

    # Re-run curl with -i to get headers
    local last_curl_cmd="${LAST_CURL_CMD}"
    curl -s -i "${last_curl_cmd[@]}" > "$response_file" 2>&1

    if grep -i "^${header_name}:" "$response_file" | grep -q "$expected_value"; then
        echo -e "${GREEN}✓ $test_name PASSED: Header '$header_name' contains '$expected_value'${NC}"
        TEST_RESULTS["$test_name"]="PASS"
        return 0
    else
        echo -e "${RED}✗ $test_name FAILED: Header '$header_name' does not contain '$expected_value'${NC}"
        echo "Response headers:"
        grep -i "^${header_name}:" "$response_file" || echo "(header not found)"
        record_test_result "$test_name" "FAIL"
        return 1
    fi
}

# Print test summary
print_summary() {
    local policy_name="$1"

    print_section "Test Summary - $policy_name"

    local pass_count=0
    local fail_count=0

    # Parse parallel arrays (bash 3.2 compatible)
    if [ -n "$TEST_RESULTS_NAMES" ]; then
        IFS='|' read -ra names <<< "$TEST_RESULTS_NAMES"
        IFS='|' read -ra statuses <<< "$TEST_RESULTS_STATUS"

        for i in "${!names[@]}"; do
            local test_name="${names[$i]}"
            local result="${statuses[$i]}"
            if [ "$result" = "PASS" ]; then
                echo -e "${GREEN}✓${NC} $test_name: PASS"
                pass_count=$((pass_count + 1))
            else
                echo -e "${RED}✗${NC} $test_name: FAIL"
                fail_count=$((fail_count + 1))
            fi
        done
    fi

    echo ""
    echo "Total: $((pass_count + fail_count)) tests"
    echo "Passed: $pass_count"
    echo "Failed: $fail_count"
    echo ""

    if [ $fail_count -eq 0 ]; then
        echo -e "${GREEN}All tests PASSED!${NC}"
        return 0
    else
        echo -e "${RED}Some tests FAILED!${NC}"
        return 1
    fi
}

# Run a simple test (just check that curl succeeds)
# Usage: run_simple_test <test_name> <curl_args...>
run_simple_test() {
    local test_name="$1"
    shift

    execute_curl "$test_name" "$@"
    validate_status "$test_name" "200"
}

# Run a test expecting rejection
# Usage: run_rejection_test <test_name> <curl_args...>
run_rejection_test() {
    local test_name="$1"
    shift

    execute_curl "$test_name" "$@"
    validate_status "$test_name" "400" "401" "403" "429"
}

# Run a test expecting specific status
# Usage: run_status_test <test_name> <expected_status> <curl_args...>
run_status_test() {
    local test_name="$1"
    local expected_status="$2"
    shift 2

    execute_curl "$test_name" "$@"
    validate_status "$test_name" "$expected_status"
}
