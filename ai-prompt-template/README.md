
# AI Prompt Template Policy Example
Applies a predefined template over an OpenAI prompt request.

## Description
When an OpenAI prompt request contains the identifier `{template://<template-name>}` (where `<template-name>` is 
a placeholder for selecting a template), this policy applies the prompt's `properties` object as replacement 
value map for the variables in the template.

## Usage Example
1. The policy is configured with a template named `veterinarian-chat`, where variable names are enclosed between 
`{{` and `}}` tokens.

```yaml
---
apiVersion: gateway.mulesoft.com/v1alpha1
kind: ApiInstance
metadata:
  name: ingress-http
spec:
  address: http://0.0.0.0:8081
  services:
    upstream:
      address: http://backend
      routes:
        - config:
            destinationPath: /anything/echo/
  policies:
    - policyRef:
        name: ai-prompt-template-v1-0-impl 
        namespace: default
      config:

        # Refuse prompts asking for unknown templates.
        allowUntemplatedRequests: false

        templates:
        
            # This name will be requested by client prompts.
          - name: veterinarian-chat

            # This is the template body. Check the {{system}} and {{species}} placeholders.
            template:  |-
              {
                "messages": [
                  {
                    "role": "system",
                    "content": "You are a {{system}} expert, in {{species}} species."
                  },
                  {
                    "role": "user",
                    "content": "Describe me the {{system}} system."
                  }
                ]
              }
```

2. An incoming prompt asks for the `veterinarian-chat` template and provides values for `system` and `species` variables:

```json
{
    "prompt": "{template://veterinarian-chat}", 
    "properties": {
        "species": "falcon", 
        "system": "respiratory"
    }
}
```

3. The policy then transforms the request payload by applying the provided values on the configured template:

```json
{
    "messages": [
        {
            "role": "system",
            "content": "You are a respiratory expert, in falcon species."
        },
        {
            "role": "user",
            "content": "Describe me the respiratory system."
        }
    ]
}

```
4. For the given configuration, if a prompt asks for an unknown template, the policy will return a `400` error.
The configuration property `allowUntemplatedRequests` must be set to `true` to change this behaviour.

## Test the Policy
Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. 
The included integration test demonstrates how to mock the upstream service by using an HTTP MockServer. 
A simple AI prompt request with an template tag is sent, and the test asserts that the template was properly applied.

To begin testing:

1. Add the `registration.yaml` in the [tests configuration folder](./tests/config).

2. Execute the `test` command:

``` shell
make test
```

### Playground Testing

To use the policy in the playground:

1. Add the `registration.yaml` in the [playground configuration folder](./playground/config).

2. Execute the `run` command to begin testing:

``` shell
make run
```

3. Make requests to the Flex Gateway by using the following curl command in order to see how the template variables are applied:

```shell
curl -X POST "http://127.0.0.1:8081" \
-H "Content-Type: application/json" \
-d '{"prompt": "{template://veterinarian-chat}", "properties": {"species": "falcon", "system": "respiratory"}}'
```

4. Change the `templates` and `allowUntemplatedRequests` properties in the playground [api.yaml](./playground/config/api.yaml) 
file to test several template configurations.

5. By default the playground is configured with an echo server as backend API. You can set an actual OpenAI API by editing the `backend` 
service at playground [docker-compose.yaml](./playground/docker-compose.yaml) file.