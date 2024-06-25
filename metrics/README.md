# Metrics Policy

Use the Metrics Policy as an example of how to periodically send data to an HTTP service.

This policy collects data from every request, and each worker periodically sends the request to a metrics ingestion service.

The policy takes the following parameters:
* `metricsSink`: The url of the metrics service.
* `pushFrequency`: The frequency the worker sends the metrics.
* `maxRetries`: The number of attempts the worker makes to send the metrics if the metrics push is unsuccessful. If left empty, the worker retries until success.

To learn more about periodic functions and HTTP calls, see:
* [Configuring Delayed and Periodic Functions](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-timer).
* [Performing an HTTP Call](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-http-request).

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration Testing

This example contains an [integration test](./tests/requests.rs) to simplify its testing. To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

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
curl "https://localhost:8081"
```

Flex Gateway logs the metrics that were successfully sent:

```text
local-flex-1  | [flex-gateway-envoy][debug] wasm log main: [policy: ingress-http-metrics-v1-0-impl-1.default][api: ingress-http.default.svc] Metrics posted successfully! {"node":"c908fb87-5106-48c1-a588-3d1edbd38562","timestamp":1712258616,"methods":{"get":1},"status_codes":{"200":1}}
```