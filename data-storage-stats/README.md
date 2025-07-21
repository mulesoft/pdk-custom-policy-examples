# Data Storage Stats Policy Example

This example demonstrates how to use data storage in your custom policy to track and count API requests per client.

To learn more about data storage, see [Data Storage](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-data-storage).

## Policy Use Case

Tracking how many requests each client makes to monitor usage patterns and identify potential abuse which:

1. Identifies clients using explicit headers (client ID or API key)
2. Counts requests per client and stores the count persistently
3. Tracks when the client last made a request
4. Provides a way to retrieve all client statistics
5. Allows resetting statistics when needed

The policy uses data storage to maintain these statistics across multiple requests and API Gateway restarts, ensuring the data persists and provides valuable insights into API usage patterns.

## Policy Behavior

The policy performs the following operations:

- **Client Identification**: Extracts client ID from the `x-client-id` header
  - **Note**: If no client identification is provided, the request is rejected with a 400 error

- **Request Counting**: For each request:
  - Increments the request statistics for the identified client
  - Updates the last request timestamp
  - Stores the statistics in data storage

- **Response Headers**: Returns the following headers with each response:
  - `x-request-count`: Total number of requests from this client
  - `x-client-id`: The identified client ID
  - `x-last-request`: Timestamp of the most recent request

- **Special Operations**:
  - **Get All Stats**: Include header `x-stats: true` to retrieve statistics for all clients
  - **Reset Statistics**: Include header `x-reset-stats: true` to clear all stored statistics

## Configuration

The policy accepts the following configuration:

```json
{
  "namespace": "request-stats"
}
```

- `namespace` (optional): The namespace for storing data. Defaults to "request-stats"

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. The tests demonstrate:

- Basic request counting functionality
- Client identification from different headers
- Error handling for missing client identification
- Statistics retrieval and reset operations
- Data persistence across multiple requests

To begin testing:

1. Add the `registration.yaml` in the `./tests/config` folder.

2. Execute the `test` command:

```shell
make test
```

### Playground Testing

To test the policy:

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
        namespace: "my-api-stats"
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to Flex Gateway:

```shell
# Request with client ID
curl http://127.0.0.1:8081/api/test -H "x-client-id: client-1"

# Request without client identification (will return 400 error)
curl http://127.0.0.1:8081/api/test

# Request to get all statistics
curl http://127.0.0.1:8081/api/test -H "x-stats: true" -i

# Request to reset all statistics
curl http://127.0.0.1:8081/api/test -H "x-reset-stats: true" -i
```

6. Check the response headers to see the request statistics for each client.

**Example get all stats response:**
```
HTTP/1.1 200 OK
x-all-stats: {"client-1":{"count":2,"last_request":1753114570},"client-2":{"count":4,"last_request":1753114578}}
date: Mon, 21 Jul 2025 16:16:28 GMT
server: Anypoint Flex Gateway
content-length: 0
```