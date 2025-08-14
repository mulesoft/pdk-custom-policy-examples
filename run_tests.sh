#!/bin/bash

# Run make test for all examples

PROJECT_DIR="$(pwd)"

echo "PDK Custom Policy Examples tests starting..."

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

            if make test 2>&1; then
                success=$((success + 1))
            else
                failed=$((failed + 1))
            fi

            cd "$PROJECT_DIR"
        fi
    fi
done

echo ""
echo "$success/$total successful, $failed failed"
echo "" 