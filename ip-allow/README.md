# IP Allow Policy

This policy filters incoming requests allowing only specified IP addresses or CIDR ranges in a configured header.

To learn more about IP filtering, see [IP Filtering](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-ip-filter).

IP Allow policy implementation:

1. The policy intercepts each incoming request and extracts the client IP from a configurable header.
2. The policy checks if the IP address matches any of the allowed IPs or CIDR ranges.
3. If the IP is allowed, the request proceeds to the upstream service.
4. If the IP is not allowed or the header is missing, the policy returns a 403 Forbidden response.

## Policy Configuration

The policy accepts the following parameters:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `ips` | array[string] | Yes | List of allowed IPs or CIDR ranges (e.g., `192.168.1.0/24`, `10.0.0.1`) |
| `ipHeader` | string | Yes | Header name from which to extract the client IP (e.g., `x-real-ip`) |

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

To test the policy in the playground:

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
        name: ip-allow-flex-v1-0
        namespace: default
      config:
        ips:
          - "192.168.1.0/24"
          - "10.0.0.1"
        ipHeader: "x-real-ip"
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to test the IP filtering:

```shell
# Test with an allowed IP (from CIDR range) - should succeed
curl -H "x-real-ip: 192.168.1.100" http://localhost:8081/anything/echo/

# Test with specific allowed IP - should succeed
curl -H "x-real-ip: 10.0.0.1" http://localhost:8081/anything/echo/

# Test with a forbidden IP - should return 403
curl -H "x-real-ip: 8.8.8.8" http://localhost:8081/anything/echo/

# Test without IP header - should return 403
curl http://localhost:8081/anything/echo/
```
