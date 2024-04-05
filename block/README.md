# Metrics Policy

Use the Metrics Policy as an example of how to create a task that is executed periodically in a single worker.

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

3. Make requests to the Flex Gateway by using the following Curl commands:

```shell
 curl http://127.0.0.1:8081/ -v -H "ip: 24.152.57.0"
 curl http://127.0.0.1:8081/ -v -H "ip: 25.152.57.0"
```

You should see that the first request was rejected and the second was completed successfully.