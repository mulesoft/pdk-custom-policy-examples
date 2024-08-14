# CORS Validation Policy

Use the CORS Validation Policy as an example of how to validate Cross Origins requests.
This policy is a simplifed variation of the [Flex CORS Included Policy](https://docs.mulesoft.com/gateway/latest/policies-included-cors), and takes the same configuration parameters.

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

3. Make requests to the Flex Gateway by using the following CURL command:

```shell
curl -H "Origin: http://www.the-origin.com"  "https://localhost:8081"
```

4. Check the Flex Gateway logs to find how many retries each request took before being accepted or rejected.
