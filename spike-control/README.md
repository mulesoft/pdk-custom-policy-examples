# Spike Control Policy Example

This policy limits how many requests reach the backend in a time window, with optional delays and retries when the limit is reached.

To learn more about spike control, see [Configuring Spike Control](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-spike-control).

Spike Control policy implementation:

1. Each incoming request is evaluated against a per-bucket quota for the configured time window.
2. If the request is within quota, it proceeds to the upstream service.
3. If the quota is exceeded and `maxAttempts` is zero, the policy responds immediately (for example HTTP 429).
4. If `maxAttempts` is greater than zero, the policy can delay and retry according to `delay` and `maxAttempts`.

Spike limits apply per worker. For example, if you set `requests` to `100` and the gateway runs two workers, up to 200 requests can be served during the window (behavior follows the runtime).

## Policy Configuration

The policy accepts the following parameters:

- requests: The number of requests that can reach the backend in the given time window.
- millis: The duration in milliseconds of the time window.
- maxAttempts: The maximum number of throttling attempts before the request is rejected (use `0` to reject immediately when over quota, with no queue).
- delay: The delay in milliseconds between each throttled attempt when `maxAttempts` is greater than zero.

## Test the Policy

Test the policy using unit tests or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Unit tests

This example contains [unit tests](./src/lib.rs) to simplify its testing.

To begin testing execute the `test` command:

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
        name: spike-control-v1-0-impl
        namespace: default
      config:
        requests: 10
        millis: 5000
        maxAttempts: 3
        delay: 500
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to the Flex Gateway:

```shell
curl -i "http://localhost:8081"
```

To see spike behavior, execute consecutive requests with 200 status code until exceeding the configured limit. After that, the gateway responds with HTTP 429.
