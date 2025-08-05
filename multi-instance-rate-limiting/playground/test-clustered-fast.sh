#!/bin/bash

# Fast test script for clustered rate limiting behavior
# This script demonstrates that two Flex instances share rate limit state

echo "🚀 Testing Multi-Instance Rate Limiting (Fast Test)"
echo "=================================================="
echo ""

# Test configuration
API_KEY="test-key-456"
INSTANCE1_URL="http://localhost:8081/anything/echo/"
INSTANCE2_URL="http://localhost:8082/anything/echo/"

echo "📋 Configuration:"
echo "  - Rate limit: 3 requests per 60 seconds"
echo "  - API Key: $API_KEY"
echo "  - Instance 1: $INSTANCE1_URL"
echo "  - Instance 2: $INSTANCE2_URL"
echo ""

echo "🧪 Testing Clustered Rate Limiting (Fast Requests)..."
echo ""

# Function to make a request and show the result
make_request() {
    local instance=$1
    local url=$2
    local request_num=$3
    
    echo -n "🔍 Request $request_num to $instance... "
    response=$(curl -s -w "\n%{http_code}" -H "x-api-key: $API_KEY" "$url")
    http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" = "200" ]; then
        echo "✅ SUCCESS"
    elif [ "$http_code" = "429" ]; then
        echo "❌ RATE LIMITED"
    else
        echo "⚠️  UNEXPECTED (HTTP $http_code)"
    fi
}

# Make requests quickly to demonstrate rate limiting
make_request "Instance 1" "$INSTANCE1_URL" "1"
make_request "Instance 1" "$INSTANCE1_URL" "2"
make_request "Instance 2" "$INSTANCE2_URL" "3"
make_request "Instance 1" "$INSTANCE1_URL" "4"
make_request "Instance 2" "$INSTANCE2_URL" "5"
make_request "Instance 1" "$INSTANCE1_URL" "6"
make_request "Instance 2" "$INSTANCE2_URL" "7"

echo ""
echo "🎯 Expected Results:"
echo "  - Requests 1-3: Should succeed (shared rate limit)"
echo "  - Requests 4-7: Should be rate limited (429 status)"
echo ""
echo "✅ Fast clustered rate limiting test completed!" 