# JSON Validation Policy Example

This policy validates incoming request bodies as JSON and optionally enforces structural limits (nesting depth, array length, object size, string and key lengths) using the PDK JSON validator library.

To learn more about JSON validation, see [Configuring JSON Validation](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-json-validation).

JSON Validation policy implementation:

1. If the request has no body, validation is skipped and the request proceeds to the upstream service.
2. If a body is present, the policy buffers the payload and validates it as JSON.
3. If validation succeeds, the request proceeds to the upstream service.
4. If the JSON is invalid or exceeds a configured limit, the policy responds with HTTP 400.

## Policy Configuration

The policy accepts the following parameters. All are optional: omit a field, use `null`, or use 0 for no limit on that constraint. With every limit disabled, only invalid JSON syntax is rejected.

- maxDepth (optional): Maximum nesting depth of JSON objects and arrays.
- maxArrayLength (optional): Maximum number of elements in a JSON array.
- maxStringLength (optional): Maximum length of JSON string values.
- maxObjectEntries (optional): Maximum number of entries in a JSON object.
- maxKeyLength (optional): Maximum length of object key names.

## Test the Policy

Test the policy using unit tests or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Unit tests

This example contains [unit tests](./src/lib.rs) to simplify its testing.

To begin testing execute the `test` command:

```shell
make test
```

### Playground Testing

To test the policy in the playground:

1. Run the `build` command to compile the policy:

```shell
make build
```

2. Configure the `playground/config/api.yaml` as follows:

```yaml
# Copyright 2023 Salesforce, Inc. All rights reserved.
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
        name: json-validation-v1-0-impl
        namespace: default
      config:
        maxDepth: 64
        maxArrayLength: 10000
        maxStringLength: 1048576
        maxObjectEntries: 10000
        maxKeyLength: 256
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to test the JSON validation:

```shell
# POST with valid JSON should succeed
curl -i -X POST "http://localhost:8081/" \
  -H "Content-Type: application/json" \
  -d '{"item":"book","qty":1}'

# POST with invalid JSON should return 400
curl -i -X POST "http://localhost:8081/" \
  -H "Content-Type: application/json" \
  -d '{"a":1]'

# GET without body should succeed as validation is skipped
curl -i "http://localhost:8081/"
```
