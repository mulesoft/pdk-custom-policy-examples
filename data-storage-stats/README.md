# Data Storage Stats Policy

This policy demonstrates how to track and manage request statistics for different clients using both local and remote storage with CAS (Compare-And-Swap) operations for thread safety.

> **Note**: For demonstration purposes, all endpoints (both "user" and "admin") are exposed without authorization controls. In production environments, proper authentication and authorization mechanisms should be implemented to secure endpoints.

## Features

- **Client Request Tracking**: Counts requests per client and tracks last request timestamp
- **Storage Options**: Supports both local (in-memory) and remote (distributed) storage
- **CAS Operations**: Uses Compare-And-Swap for thread-safe concurrent updates
- **RESTful API**: Provides endpoints to retrieve and reset statistics
- **Configurable Retries**: Handles storage conflicts with configurable retry logic
- **Thread Safety**: Ensures accurate counting under high concurrency

## Configuration example

```json
{
  "namespace": "request-stats",
  "storage_type": "remote",
  "ttl_seconds": 3600,
  "max_retries": 3
}
```

### Configuration Parameters

- `namespace` (string, default: "request-stats"): Namespace to isolate data between policy instances
- `storage_type` (string, default: "local"): Storage type - "local" for in-memory or "remote" for distributed
- `ttl_seconds` (number, default: 60): Time-to-live for stored items in seconds (remote storage only)
- `max_retries` (number, required): Maximum number of retries for CAS operations

## API Endpoints

### Client Operations

Any request path (except admin endpoints) will increment the client's request counter.

**Required Header:**
- `x-client-id`: Client identifier (must not be empty)

**Response:** The request continues to the backend without any statistics headers. Statistics are stored internally and are only accessible via admin endpoints.

**Example:**
```bash
curl -H "x-client-id: client-123" http://localhost:8081/api/resource
```

### Admin Operations

#### GET /stats
Retrieves all client statistics.

**Response:**
- Status: 200 OK
- Content-Type: application/json
- Body: JSON object with client IDs as keys and stats as values

```json
{
  "client-123": {
    "count": 5,
    "last_request": 1703123456
  },
  "client-456": {
    "count": 2,
    "last_request": 1703123400
  }
}
```

#### DELETE /stats
Resets all client statistics.

**Response:**
- Status: 200 OK
- Content-Type: application/json
- Body: Confirmation message with timestamp

```json
{
  "message": "All statistics have been reset successfully",
  "timestamp": 1703123456
}
```

## Testing

### Integration tests

1. Add the `registration.yaml` in the `./tests/config` folder.

2. Execute the `test` command:

```shell
make test
```

### Playground Testing

The playground is configured to use Redis as the remote storage backend:

- **Redis**: Running on port 6379 with persistent storage
- **Flex Gateway**: Running on port 8081
- **Backend**: HTTPBin service for testing
- **Storage Type**: Remote (Redis)

#### Configuration Files

- `docker-compose.yaml`: Defines Redis, Flex Gateway, and backend services
- `config/shared-storage-redis.yaml`: Redis configuration for Flex Gateway
- `config/api.yaml`: API configuration with remote storage settings
- `config/custom-policies/`: Policy definition and implementation

#### Storage Configuration

The policy is configured to use:
- **Storage Type**: Remote (Redis)
- **TTL**: 3600 seconds (1 hour)
- **Max Retries**: 3 for CAS operations
- **Namespace**: "request-stats"

#### Start the services

1. Run the `build` command to compile the policy:

```shell
make build
```

2. Configure the `playground/config/api.yaml` as follows:

```yaml
# Copyright 2023 Salesforce, Inc. All rights reserved.
---
apiVersion: gateway.mulesoft.com/v1alpha1
kind: ApiInstance
metadata:
  name: ingress-http
spec:
  address: http://0.0.0.0:8081
  services:
    upstream:
      address: http://backend
      routes:
        - config:
            destinationPath: /anything/echo/
  policies:
    - policyRef:
        name: data-storage-stats-v1-0-impl
      config:
        namespace: "request-stats"
        storage_type: "remote"
        ttl_seconds: 3600
        max_retries: 3
        shared_storage: "shared-storage-redis"
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

This will:
1. Build the policy implementation
2. Start Redis, Flex Gateway and the backend service
3. Configure the policy to use remote storage (Redis)

#### Test

Once the services are running, you can test the policy manually by making requests:

**Basic requests with client tracking:**
```shell
# Request from client-1
curl -H "x-client-id: client-1" http://localhost:8081/test

# Request from client-2
curl -H "x-client-id: client-2" http://localhost:8081/test

# Another request from client-1
curl -H "x-client-id: client-1" http://localhost:8081/test
```

**Admin operations:**
```shell
# Get all stats (returns JSON in response body)
curl http://localhost:8081/stats
```

**Example Output:**
```json
{
  "client-2": {
    "count": 1,
    "last_request": 1753300876
  },
  "client-1": {
    "count": 2,
    "last_request": 1753300862
  }
}
```

```shell
# Reset all stats (DELETE method)
curl -X DELETE http://localhost:8081/stats
```

**Example Output:**
```json
{
  "message": "All statistics have been reset successfully",
  "timestamp": 1753300876
}
```

```shell
# Verify stats are cleared
curl http://localhost:8081/stats
```

**Example Output:**
```json
{}
```

#### Redis Inspection

You can inspect Redis data directly:

```bash
# Connect to Redis CLI
docker exec -it $(docker ps -q -f name=redis) redis-cli

# List all keys
KEYS *

# Get a specific key value
GET "request-stats:client-1"

# Check TTL for a key
TTL "request-stats:client-1"

# Monitor Redis operations in real-time
MONITOR
```

#### CAS retry testing

```bash
# Simulate high concurrency to test CAS retry logic
for i in {1..10}; do
  curl -H "x-client-id: cas-test-client" http://localhost:8081/test &
done
wait

# Verify final count is correct
curl http://localhost:8081/stats | jq '.["cas-test-client"].count'
# Should return: 10
```