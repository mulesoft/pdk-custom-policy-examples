# Copyright 2023 Salesforce, Inc. All rights reserved.

#!/bin/bash

# Run make test for all examples

PROJECT_DIR="$(pwd)"
FAILURES_LOG="$PROJECT_DIR/failures.log"

echo "PDK Custom Policy Examples tests starting..."
: > "$FAILURES_LOG"

total=0
success=0
failed=0

# Process each example directory
for dir in */; do
    if [[ -d "$dir" && ! "$dir" =~ ^\. ]]; then
        dir_name="${dir%/}"
        
        # Check if it has Makefile and Cargo.toml
        if [[ -f "$dir_name/Makefile" && -f "$dir_name/Cargo.toml" ]]; then
            total=$((total + 1))

            cd "$dir_name"

            echo ""
            echo "Testing $dir_name..."
            echo ""

            make setup

            tmp_out="$(mktemp)"

            make test 2>&1 | tee "$tmp_out"

            if [[ "${PIPESTATUS[0]}" -eq 0 ]]; then
                success=$((success + 1))
            else
                failed=$((failed + 1))
                {
                    echo "========== $dir_name =========="
                    cat "$tmp_out"
                    echo ""
                } >> "$FAILURES_LOG"
            fi
            rm -f "$tmp_out"

            cd "$PROJECT_DIR"
        fi
    fi
done

echo ""
echo "$success/$total successful, $failed failed"
if [[ "$failed" -gt 0 ]]; then
    echo "Errors written to $FAILURES_LOG"
else
    rm -f "$FAILURES_LOG"
fi
echo ""