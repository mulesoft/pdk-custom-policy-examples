# Stop Iteration Policy

Use the Stop Iteration Policy as an example of how to work with request and response state iteration to access both headers and body in a single state.

This policy demonstrates how to use the `into_headers_body_state()` method to transition to a state where both headers and body can be accessed and modified simultaneously. The policy can modify requests, responses, or both, based on configuration.

The policy takes the following parameters:
* `modifyRequest`: Boolean flag to enable request modification (default: false).
* `modifyResponse`: Boolean flag to enable response modification (default: false).
* `bodyPrefix`: String prefix to add to the body content (default: "modified").

## Policy Behavior

### Request Modification
When `modifyRequest` is enabled, the policy:
1. Transitions to the headers-body state using `into_headers_body_state()`.
2. Extracts the original HTTP method from the `:method` header.
3. Adds an `x-original-method` header with the original method value.
4. Modifies the request body by prefixing it with the configured `bodyPrefix` (format: `{bodyPrefix}-{original-body}`).

### Response Modification
When `modifyResponse` is enabled, the policy:
1. Transitions to the headers-body state for the response.
2. Adds an `x-stop-iteration` header with value `response-modified`.
3. Modifies the response body by prefixing it with the configured `bodyPrefix` (format: `{bodyPrefix}:{original-body}`).

To learn more about state iteration and working with headers and bodies, see:
* [Reading and Writing Request Headers and Bodies](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-headers).

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

3. Make requests to the Flex Gateway by using the following Curl commands:

```shell
# Test with response modification only
curl http://127.0.0.1:8081/hello -v

# Test with POST request to see request body modification
curl http://127.0.0.1:8081/hello -X POST -d "test-data" -v

# Check for the x-stop-iteration header in the response
curl http://127.0.0.1:8081/hello -i
```

The policy will add the `x-stop-iteration: response-modified` header and modify the response body with the configured prefix.
