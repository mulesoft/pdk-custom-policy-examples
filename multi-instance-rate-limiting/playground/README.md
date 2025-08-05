# Multi-Instance Rate Limiting Playground

This playground demonstrates the clustered rate limiting behavior with two Flex Gateway instances sharing rate limit state through Redis.

## Architecture

- **Flex Gateway Instance 1**: Running on port 8081
- **Flex Gateway Instance 2**: Running on port 8082  
- **Backend**: httpbin service on port 8080
- **Redis**: Shared storage for rate limit state on port 6379

## Getting Started

1. Build and run the playground:
   ```bash
   make run
   ```

2. Test the clustered rate limiting behavior:
   ```bash
   # Test requests to instance 1
   curl -H "x-api-key: test-key-1" http://localhost:8081/anything/echo/
   
   # Test requests to instance 2 (should share rate limit state)
   curl -H "x-api-key: test-key-1" http://localhost:8082/anything/echo/
   ```

## Testing Clustered Behavior

The rate limit is configured for 3 requests per 60 seconds. You can test that both instances share the same rate limit state:

### Quick Test
```bash
./test-clustered-fast.sh
```

### Manual Test
1. Make 2 requests to instance 1 (should succeed)
2. Make 1 request to instance 2 (should succeed - total 3 requests)
3. Make another request to either instance (should be blocked with 429 status)

### Expected Results
- **Requests 1-3**: Should succeed (shared rate limit across instances)
- **Requests 4+**: Should be rate limited (429 status)

The playground demonstrates that rate limiting state is shared between multiple Flex Gateway instances, enabling true clustered behavior.

## Configuration

The rate limiting is configured in `config/api.yaml`:
- **Group**: "api"
- **Requests per window**: 3
- **Window size**: 60 seconds
- **Key selector**: "api_key" (uses x-api-key header)
- **Shared storage**: Redis

## Stopping the Playground

```bash
docker compose -f ./playground/docker-compose.yaml down
``` 