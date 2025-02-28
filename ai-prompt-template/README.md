
# AI Prompt Template Policy Example
Applies a template on an OpenAI prompt request.

## Test the Policy
Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. The included integration test demonstrates how to mock the upstream service by using an HTTP MockServer. A simple AI prompt request with an array of messages is sent, and the test asserts that it validated by appliying regular expressions.

To begin testing:

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

3. Make requests to the Flex Gateway by using the following Curl command in order to see how the template is applied:

```shell
curl -X POST "http://127.0.0.1:8081" \
-H "Content-Type: application/json" \
-d '{"prompt": "{template://raw-template}", "properties": {"foo": "a foo value", "bar": "a bar value"}}'
```

4. Change the `templates` property in `./playground/config/api.yaml` to test several template configurations.

5. By default the playground is configured with an echo server as backend API. You can set an actual OpenAI API by editing the `backend` service at `./playground/docker-compose.yaml` file.