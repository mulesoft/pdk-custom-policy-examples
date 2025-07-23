# Data Storage Stats Policy Example

This example demonstrates how to use data storage in your custom policy to track and count API requests per client.

## Policy Use Case

Tracks request counts per client to monitor usage patterns and identify potential abuse:

1. Identifies clients using `x-client-id` header
2. Counts requests per client persistently
3. Tracks last request timestamp
4. Provides admin operations to retrieve/reset statistics

## Policy Behavior

- **Client Identification**: Extracts client ID from `x-client-id` header
  - Rejects requests without client identification (400 error)

- **Request Counting**: Increments statistics and updates timestamp for each request

- **Response Headers**: Returns:
  - `x-request-count`: Total requests from this client
  - `x-client-id`: The client ID
  - `x-last-request`: Timestamp of most recent request

- **Admin Operations**:
  - **Get All Stats**: `GET /stats` - retrieves statistics for all clients
  - **Reset Statistics**: `POST /stats/reset` - clears all stored statistics

## Configuration

```json
{
  "namespace": "request-stats",
  "storage_type": "local",
  "max_retries": 3
}
```

- `namespace` (optional): Data storage namespace. Defaults to "request-stats"
- `storage_type` (optional): "local" or "remote". Defaults to "local"
- `max_retries` (optional): CAS retry attempts. Defaults to 3

## Testing

### Integration Tests

```shell
make test
```

### Playground Testing

1. Build the policy:
```shell
make build
```

2. Configure `playground/config/api.yaml`:
```yaml
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
        namespace: "my-api-stats"
```

3. Run the playground:
```shell
make run
```

4. Test with curl:
```shell
# Regular request
curl http://127.0.0.1:8081/api/test -H "x-client-id: client-1"

# Get all stats
curl http://127.0.0.1:8081/stats

# Reset stats
curl -X POST http://127.0.0.1:8081/stats/reset
```