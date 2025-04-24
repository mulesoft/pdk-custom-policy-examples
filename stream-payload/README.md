# Stream Payload Policy

Use the Stream Payload Policy as an example of how to read payloads that are bigger than the underlying buffer.

This policy processes the body and rejects the request if it contains a forbidden string.

The policy takes the following parameters:
* `searchMode`: The mode in which the policy will check the body
  * `streamed`: On each received chunk we check for appearances of the forbidden strings. The request will be rejected before the whole payload is received if there is a match.
  * `buffered`: Collect the whole body before checking for appearances.
* `forbiddenStrings`: A list of the strings that will be searched on the body.

To learn more about manipulating the body, see:
* [Reading and Writing Request Headers and Bodies](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-headers).

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
curl "http://localhost:8081" -d '${jndi'
```

The policy should reject the request and with the following message

```text
{ "error" : "forbidden string detected on the payload" }
```
