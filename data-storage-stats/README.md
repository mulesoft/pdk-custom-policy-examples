# Data Storage Stats Policy

This policy demonstrates how to use data storage capabilities in your custom policy.

To learn more about data storage, see [Data Storage](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-data-storage).

## Policy use case

An organization needs to track how frequently each client accesses their API and when they last made requests. The goal is to maintain per-client request statistics that persist even if the API Gateway restarts and to provide administrative capabilities for retrieving and resetting them.

Data Storage policy implementation:

1. The policy intercepts each incoming request and extracts the client identifier from the `x-client-id` header.
2. The policy updates the client's request count and timestamp using persistent storage with CAS (Compare-And-Swap) operations for thread safety.
3. The policy supports both local (in-memory) and remote (Redis) storage backends for different deployment scenarios.
4. "Admin" endpoints (`GET /stats` and `DELETE /stats`) are provided for monitoring and management.
5. The policy is designed so that any error in the statistics tracking flow does not block or fail the original request.

> **Note**: For demonstration purposes, all endpoints (both "user" and "admin") are exposed without authorization controls. In production environments, proper authentication and authorization mechanisms should be implemented to secure endpoints.

To reuse the policy, the storage type (local vs remote), namespace isolation, and retry mechanisms are configurable.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. The included integration tests cover basic functionality, admin operations, concurrency handling, remote storage, and error scenarios.

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
        namespace: default
      config:
        namespace: "request-stats"
        storage_type: "remote"
        ttl_seconds: 3600
        max_retries: 3
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to Flex Gateway:

```shell
# Test client request tracking
curl http://127.0.0.1:8081/test -H "x-client-id: client-123"

# Retrieve all statistics
curl http://127.0.0.1:8081/stats

# Reset all statistics
curl -X DELETE http://127.0.0.1:8081/stats
```

If you repeat the request with the same client ID, you should see that the request count increases in the statistics response. The data is persisted in storage and will survive API Gateway restarts.
