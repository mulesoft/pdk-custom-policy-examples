#!/bin/bash

# test-all-playgrounds.sh - Master test script for all playground tests
# Usage:
#   ./test-all-playgrounds.sh           # Run all tests
#   ./test-all-playgrounds.sh block     # Run specific test
#   ./test-all-playgrounds.sh --list    # List all available tests

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Find all directories with test-playground.sh
find_test_dirs() {
    find "$SCRIPT_DIR" -maxdepth 2 -name "test-playground.sh" -type f | \
        xargs -I {} dirname {} | \
        xargs -I {} basename {} | \
        sort
}

# List available tests
list_tests() {
    echo "Available playground tests:"
    echo ""
    find_test_dirs | while read dir; do
        echo "  - $dir"
    done
    echo ""
    echo "Usage:"
    echo "  ./test-all-playgrounds.sh              # Run all tests"
    echo "  ./test-all-playgrounds.sh <policy>     # Run specific test"
    echo "  ./test-all-playgrounds.sh --list       # List all tests"
}

# Run a single test
run_test() {
    local dir="$1"
    local test_script="$SCRIPT_DIR/$dir/test-playground.sh"

    if [ ! -f "$test_script" ]; then
        echo -e "${RED}Error: Test script not found: $test_script${NC}"
        return 1
    fi

    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}Testing: $dir${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo ""

    cd "$SCRIPT_DIR/$dir"

    if bash "$test_script"; then
        echo -e "${GREEN}✓ $dir tests PASSED${NC}"
        return 0
    else
        echo -e "${RED}✗ $dir tests FAILED${NC}"
        return 1
    fi
}

# Main function
main() {
    local target="$1"

    # Handle --list flag
    if [ "$target" = "--list" ] || [ "$target" = "-l" ]; then
        list_tests
        exit 0
    fi

    # Handle --help flag
    if [ "$target" = "--help" ] || [ "$target" = "-h" ]; then
        list_tests
        exit 0
    fi

    # If specific test is requested
    if [ -n "$target" ]; then
        echo -e "${BLUE}Running single test: $target${NC}"
        if run_test "$target"; then
            exit 0
        else
            exit 1
        fi
    fi

    # Run all tests
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Running All Playground Tests          ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""

    local total=0
    local passed=0
    local failed=0
    declare -a failed_tests

    while read dir; do
        total=$((total + 1))
        if run_test "$dir"; then
            passed=$((passed + 1))
        else
            failed=$((failed + 1))
            failed_tests+=("$dir")
        fi
        echo ""
    done < <(find_test_dirs)

    # Print summary
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║           Test Summary                  ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "Total:  $total"
    echo -e "${GREEN}Passed: $passed${NC}"
    if [ $failed -gt 0 ]; then
        echo -e "${RED}Failed: $failed${NC}"
        echo ""
        echo "Failed tests:"
        for test in "${failed_tests[@]}"; do
            echo -e "  ${RED}✗${NC} $test"
        done
    fi
    echo ""

    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}All tests PASSED! 🎉${NC}"
        exit 0
    else
        echo -e "${RED}Some tests FAILED${NC}"
        exit 1
    fi
}

# Run main function
main "$@"
