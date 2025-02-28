# AI Prompt Decorator Policy Example
Use the basic PDK body manipulation functionallities to implement an example of a message decorator policy for an OpenAI API.

## Policy use case
The AI Prompt Decorator policy example preppends and/or appends an array of messages to an OpenAI API consumer chat history. 
This allows for the advance crafting of intricate prompts or the guiding (and protecting) of prompts so that any changes made to the consumer’s message within the LLM remain entirely transparent.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. In the included integration tests demonstrate how to mock the upstream service by using an HTTP MockServer. A simple chat request with an array of messages is sent, and the test asserts that it is decorated with the configured prepend and append messages.

To begin testing:

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
curl -X POST "http://127.0.0.1:8081" \
-H "Content-Type: application/json" \
-d '{"model": "llama", "messages": [{"role": "user", "content": "Give me an example of exo-planet"}]}'
```

Flex Gateway should return a response with the query parameters as headers:

```json
{
  "args": {}, 
  "data": "{\"model\":\"llama\",\"messages\":[{\"role\":\"system\",\"content\":\"You are astronomer.\"},{\"role\":\"user\",\"content\":\"Focus on solar system.\"},{\"role\":\"user\",\"content\":\"Give me an example of exo-planet\"},{\"role\":\"user\",\"content\":\"Do not use speculative theories.\"}]}", 
  "files": {}, 
  "form": {}, 
  "headers": {
    "Accept": "*/*", 
    "Content-Type": "application/json", 
    "Host": "backend", 
    "Transfer-Encoding": "chunked", 
    "X-Envoy-Expected-Rq-Timeout-Ms": "15000", 
    "X-Envoy-Internal": "true", 
    "X-Envoy-Original-Path": "/"
  }, 
  "json": {
    "model": "llama",
    "messages": [
      {
        "content": "You are astronomer.", 
        "role": "system"
      }, 
      {
        "content": "Focus on solar system.", 
        "role": "user"
      }, 
      {
        "content": "Give me an example of exo-planet", 
        "role": "user"
      }, 
      {
        "content": "Do not use speculative theories.", 
        "role": "user"
      }
    ]
  }, 
  "method": "POST", 
  "url": "http://backend/anything/echo/"
}

```

4. Change the `prepend` and `append` messages in `./playground/config/api.yaml` to test several chat history decorations.

5. By default the playground is configured with an echo server as backend API. You could set an actual OpenAI API by editing the `backend` service at `./playground/docker-compose.yaml` file.
