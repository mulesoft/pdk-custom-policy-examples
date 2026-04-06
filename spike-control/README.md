# Spike Control Policy

Use the Spike Control Policy as an example of how to limit how many requests reach the backend in a time window, with optional delays and retries when the limit is reached.

Spike limits apply per worker. For example, if you set `requests` to `100` and the gateway runs two workers, up to 200 requests can be served during the window (behavior follows the runtime).

The policy takes the following parameters:

* `requests`: The number of requests that can reach the backend in the given time window.
* `millis`: The duration in milliseconds of the time window.
* `maxAttempts`: The maximum number of throttling attempts before the request is rejected (use `0` to reject immediately when over quota, with no queue).
* `delay`: The delay in milliseconds between each throttled attempt when `maxAttempts` is greater than zero.

To learn more about spike control, see [Configuring Spike Control](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-spike-control).

## Test the Policy

Test the policy using unit tests, integration tests, or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Unit tests

```shell
cargo test --lib
```

### Integration tests

This example contains an [integration test](./tests/requests.rs). The test uses the policy implementation name from `target/policy-ref-name.txt`, which **`make build`** generates together with the WASM and implementation GCL. Use **`make test`** so the full build runs before Docker-based tests.

To begin testing:

1. Add `registration.yaml` in the `./tests/config` folder.

2. Execute the `test` command:

```shell
make test
```

### Playground testing

To test the policy in the playground:

1. Run the `build` command to compile the policy:

```shell
make build
```

2. Configure `playground/config/api.yaml` as follows (set `policyRef.name` to the value printed by `make show-policy-ref-name`; `make run` overwrites it automatically):

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

3. Place `registration.yaml` in `playground/config`.

4. Run the gateway:

```shell
make run
```

5. Send requests to the Flex Gateway.


```shell
curl -v "http://localhost:8081"
```

To see spike behavior, execute consecutive requests with 200 status code until exceeding the configured limit. After that, the gateway responds with HTTP 429.
