#!/bin/bash

# Test script for clustered rate limiting behavior
# This script demonstrates that two Flex instances share rate limit state

echo "🚀 Testing Multi-Instance Rate Limiting (Clustered Behavior)"
echo "=========================================================="
echo ""

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 5

# Test configuration
API_KEY="test-key-123"
INSTANCE1_URL="http://localhost:8081/anything/echo/"
INSTANCE2_URL="http://localhost:8082/anything/echo/"

echo "📋 Configuration:"
echo "  - Rate limit: 3 requests per 60 seconds"
echo "  - API Key: $API_KEY"
echo "  - Instance 1: $INSTANCE1_URL"
echo "  - Instance 2: $INSTANCE2_URL"
echo ""

# Function to make a request and show the result
make_request() {
    local instance=$1
    local url=$2
    local request_num=$3
    
    echo "🔍 Request $request_num to $instance..."
    response=$(curl -s -w "\n%{http_code}" -H "x-api-key: $API_KEY" "$url")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        echo "✅ SUCCESS (HTTP $http_code)"
    elif [ "$http_code" = "429" ]; then
        echo "❌ RATE LIMITED (HTTP $http_code)"
    else
        echo "⚠️  UNEXPECTED (HTTP $http_code)"
    fi
    echo ""
}

echo "🧪 Testing Clustered Rate Limiting..."
echo ""

# Request 1: Instance 1 (should succeed)
make_request "Instance 1" "$INSTANCE1_URL" "1"

# Request 2: Instance 1 (should succeed)  
make_request "Instance 1" "$INSTANCE1_URL" "2"

# Request 3: Instance 2 (should succeed - shared state)
make_request "Instance 2" "$INSTANCE2_URL" "3"

# Request 4: Instance 1 (should be rate limited)
make_request "Instance 1" "$INSTANCE1_URL" "4"

# Request 5: Instance 2 (should be rate limited)
make_request "Instance 2" "$INSTANCE2_URL" "5"

echo "🎯 Test Summary:"
echo "  - Requests 1-3 should succeed (shared rate limit)"
echo "  - Requests 4-5 should be rate limited (429 status)"
echo ""
echo "✅ Clustered rate limiting test completed!" 