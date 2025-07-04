# Spike Policy

Use the Spike Policy as an example of how to delay requests before they reach the backend.

This policy provides spike control for each worker. For example, if you set `requests` to `100` and have two workers, 200 requests are serviced during the time window.

The policy takes the following parameters:
* `requests`: The amount of requests that can reach the backend service in the given time window.
* `millis`: The duration in milliseconds of the time window.
* `maxAttempts`: The maximum number of attempts the request is throttled for before it is rejected.
* `delay`: The delay in milliseconds between each throttled attempt.

To learn more about periodic functions, see [Configuring Delayed and Periodic Functions](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-timer).

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration Testing

This example contains an [integration test](./tests/requests.rs) to simplify its testing. To begin testing:

1. Add the `registration.yaml` in the `./tests/config` folder.

2. Execute the `test` command:

``` shell
make test
```

### Playground Testing

To use the policy in the playground:

1. Add the `registration.yaml` in the `./playground/config` folder

2. Execute the `run` command to begin testing:

``` shell
make run
```

3. Make requests to the Flex Gateway by using the following Curl command:

```shell
curl "http://localhost:8081"
```

4. View the Flex Gateway logs to find how many retries each request took before being accepted or rejected, for example:

```text
local-flex-1  | [flex-gateway-envoy][debug] wasm log ingress-http-spike-v1-0-impl-1.default.ingress-http.default.svc main: [policy: ingress-http-spike-v1-0-impl-1.default][api: ingress-http.default.svc][req: 41dfe989-3ee2-4995-b509-0e740204f7b8] Retries: 0
local-flex-1  | [flex-gateway-envoy][debug] wasm log ingress-http-spike-v1-0-impl-1.default.ingress-http.default.svc main: [policy: ingress-http-spike-v1-0-impl-1.default][api: ingress-http.default.svc][req: 1c15995c-a29a-4ea2-a466-dbe7d05f0f8f] Retries: 3
```