# AI Basic Token Rate Limiting Policy Example
Validates the `llm/v1/chat` messages by counting tokens on a time limit basis.

## Policy use case
An `llm/v1/chat` API wants to limit incoming tokens in order to prevent token flooding.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. In the included integration tests demonstrate how to mock the upstream service by using an HTTP MockServer. A simple chat request with an array of messages is sent, and the test asserts that it is decorated with the configured prepend and append messages.

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

3. Make requests to the Flex Gateway by using the following Curl command many times in order to get a token rate limiting validation message:

```shell
curl -X POST "http://127.0.0.1:8081" \
-H "Content-Type: application/json" \
-d '{"model": "Llama", "messages": [{"role": "user", "content": "Give me an example of planet"}]}'
```

In the first hit, Flex Gateway should return a response with the echo from the backend. 
In the third hit, it should inform the token rate limit validation by returning a 403 status.

4. Change the `maximumTokens` and `timePeriodInMilliseconds` in `./playground/config/api.yaml` to test several token rate limit configurations.

5. By default the playground is configured with an echo server as backend API. You could set an actual `/llm/v1/chat` by editing the `backend` service at `./playground/docker-compose.yaml` file.
