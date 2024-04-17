# Block Policy

Use the Block Policy as an example of how to execute a task periodically in a single worker and then share the information with other workers.

This policy periodically queries a service that returns a list of IP ranges and then blocks all requests coming from those ranges. Each worker first ensures that the IP does not need to be requested. If it does, the worker makes a request to the IP source and then shares the data with the other workers.

The policy takes the following parameters:
* `source`: The url of the service that provides the list of IP ranges to block.
* `frequency`: The frequency in seconds that the service is queried.
* `ip`: A DataWeave expression that extracts the IP address from the request.

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

The policy rejects the first request and successfully completes the second.
