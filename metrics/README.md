# Metrics Policy

Use the Metrics Policy as an example of how to create a task that is executed periodically.

This policy collects data from every request, and each worker periodically sends the request to a metrics ingestion service.

The policy takes the following parameters:
* metricsSink: The url of the service that will ingest the metrics generated by the policy.
* pushFrequency: The frequency with will be used to push the metrics.
* maxRetries: If the metrics pushing fails, the amount of retries that will be made before discarding the metrics. If left empty it will retry until success.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration Testing

This example contains an [integration test](./tests/requests.rs) to simplify its testing. To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

The test can be invoked by using the `test` command:
2. Execute the `test` command:

``` shell
make test
```

### Playground Testing

To use the policy in the playground:

1. Add the `registration.yaml` in the `./payground/config` folder

2. Execute the `run` command to begin testing:

``` shell
make run
```

3. Make requests to the Flex Gateway by using the following Curl command:

```shell
curl "https://localhost:8081"
```
Flex Gateway should log the metrics that were successfully sent:

```text
local-flex-1  | [flex-gateway-envoy][debug] wasm log main: [policy: ingress-http-metrics-v1-0-impl-1.default][api: ingress-http.default.svc] Metrics posted successfully! {"node":"c908fb87-5106-48c1-a588-3d1edbd38562","timestamp":1712258616,"methods":{"get":1},"status_codes":{"200":1}}
```