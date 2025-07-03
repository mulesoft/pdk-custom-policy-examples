# Dataweave Evaluation Policy

Use the Dataweave Evaluation Policy as an example of how to evaluate custom Dataweave expressions in your custom policy.

This policy receives a Dataweave expression as part of the configuration and binds the different elements to complete its resolution.
The result of the resolution is sent as an HTTP response.

The policy takes the following parameters:
* `expression`: The Dataweave expression to evaluate.

To learn more about Dataweave resolution check 
* [Using DataWeave Expressions](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-dataweave).

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
curl "http://localhost:8081"  -u "user:pass"
```

The policy should respond with the configured expression result

```text
{"result":["user","pass"]}
```
