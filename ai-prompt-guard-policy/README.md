
# AI Prompt Guard Policy Example
Sanitizes an OpenAI chat request by refusing or deleting messages after appliying regular expression filters.

## Policy use case
An OpenAI API wants to delete or refuse incoming messages in order to prevent misuse.

## Test the Policy
Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. The included integration test demonstrates how to mock the upstream service by using an HTTP MockServer. A simple chat request with an array of messages is sent, and the test asserts that it validated by appliying regular expressions.

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

3. Make requests to the Flex Gateway by using the following Curl command in order to see how the email token is deleted:

```shell
curl -X POST "http://127.0.0.1:8081" \
-H "Content-Type: application/json" \
-d '{"model": "llama", "messages": [{"role": "user", "content": "My email es flexmaster@salesforce.com"}]}'
```

4. Make a new request to check that phone numbers are refused by returning a 403 status.
```shell
curl -X POST "http://127.0.0.1:8081" \
-H "Content-Type: application/json" \
-d '{"model": "llama", "messages": [{"role": "user", "content": "My email es flexmaster@salesforce.com and my phone number is +1 9343 6126649"}]}'
```

4. Change the `pattern` and `omitInsteadOfBlocking` properties in `./playground/config/api.yaml` to test several guards.

5. By default the playground is configured with an echo server as backend API. You can set an actual OpenAI API by editing the `backend` service at `./playground/docker-compose.yaml` file.
