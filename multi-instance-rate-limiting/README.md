# Multi-Instance Rate Limiting Policy

This policy demonstrates how to implement rate limiting capabilities that can be shared across multiple gateway instances.

To learn more about rate limiting, see [Rate Limiting](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-rate-limiting).

## Policy use case

An organization needs to implement rate limiting across multiple Flex Gateway instances to ensure consistent rate limiting behavior regardless of which instance receives the request.

Multi-Instance Rate Limiting example policy implementation:

1. The policy intercepts each incoming request and extracts client identifiers from `x-api-key` and `x-user-id` headers.
2. The policy applies rate limits only for headers that are present.
3. The policy uses clustered mode with shared storage (Redis) to coordinate rate limiting across multiple gateway instances.
4. Rate limit state is persisted in Redis, ensuring consistency across all instances.
5. The policy supports two independent rate limit configurations: API key-based and user ID-based, each with their own limits and time windows.
6. The policy is designed so that any error in the rate limiting flow blocks the request.

> **Note**: This policy requires Redis to be configured as shared storage for multi-instance coordination. For single-instance deployments, the policy will still work but won't provide cross-instance rate limiting.

Details on how to enable Shared Storage at https://docs.mulesoft.com/gateway/latest/flex-conn-shared-storage-config.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains [integration tests](./tests/requests.rs) to simplify its testing.

To begin testing:

1. Add the `registration.yaml` in the `./tests/config` folder.

2. Execute the `test` command:

```shell
make test
```

### Playground Testing

To test the clustered behavior with multiple instances:

1. The playground is configured with two Flex Gateway instances (ports 8081 and 8082)
2. Both instances share the same Redis backend for rate limit state
3. Rate limits are enforced across both instances

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
        name: multi-instance-rate-limiting-v1-0-impl
        namespace: default
      config:
        api_key_rate_limit:
          requests_per_window: 3
          window_size_seconds: 10
        user_id_rate_limit:
          requests_per_window: 5
          window_size_seconds: 10
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to test rate limiting across both instances:

```shell
# Test API key rate limiting on instance 1 (3 requests per 10s)
curl -H "x-api-key: key-1" http://localhost:8081/anything/echo/
curl -H "x-api-key: key-1" http://localhost:8081/anything/echo/
curl -H "x-api-key: key-1" http://localhost:8081/anything/echo/

# This should be rate limited on instance 1
curl -H "x-api-key: key-1" http://localhost:8081/anything/echo/

# Test the SAME API key on instance 2 - should also be rate limited due to shared storage
curl -H "x-api-key: key-1" http://localhost:8082/anything/echo/

# Test user ID rate limiting on instance 2 (5 requests per 10s)
curl -H "x-user-id: user-1" http://localhost:8082/anything/echo/
curl -H "x-user-id: user-1" http://localhost:8082/anything/echo/
curl -H "x-user-id: user-1" http://localhost:8082/anything/echo/
curl -H "x-user-id: user-1" http://localhost:8082/anything/echo/
curl -H "x-user-id: user-1" http://localhost:8082/anything/echo/

# This should be rate limited on instance 2
curl -H "x-user-id: user-1" http://localhost:8082/anything/echo/

# Test the same user ID on instance 1 - should also be rate limited
curl -H "x-user-id: user-1" http://localhost:8081/anything/echo/

# Test both headers simultaneously (both rate limits apply independently)
curl -H "x-api-key: key-2" -H "x-user-id: user-2" http://localhost:8081/anything/echo/

# Test independent rate limits (user ID still works after API key is rate limited)
curl -H "x-user-id: user-3" http://localhost:8081/anything/echo/

# Test requests without headers (should pass without rate limiting)
curl http://localhost:8081/anything/echo/
curl http://localhost:8082/anything/echo/
```

> **Note**: Rate limiting uses sliding windows so requests must be made quickly to see the expected behavior.