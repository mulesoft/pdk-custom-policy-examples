# JSON Validation Policy Example

Use the JSON Validation Policy as an example of how to validate incoming request bodies as JSON and optionally enforce structural limits (nesting depth, array length, object size, string and key lengths) using the PDK JSON validator library.

Requests without a body skip validation and are forwarded upstream. When a body is present, the policy buffers the payload, validates it and returns HTTP 400 if the JSON is invalid or exceeds a configured limit; otherwise the request continues to the backend.

## Policy Configuration

The policy takes the following parameters. All are optional: omit a field, use `null`, or use 0 for no limit on that constraint. With every limit disabled, only invalid JSON syntax is rejected.

* `maxDepth`: Maximum nesting depth of JSON objects and arrays.
* `maxArrayLength`: Maximum number of elements in a JSON array.
* `maxStringLength`: Maximum length of JSON string values.
* `maxObjectEntries`: Maximum number of entries in a JSON object.
* `maxKeyLength`: Maximum length of object key names.

To learn more about JSON validation, see [Configuring JSON Validation](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-json-validation).

## Test the Policy

Test the policy using unit tests or the policy playground.

To find the prereqs and to learn more, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests)
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Unit tests

```shell
cargo test --lib
```

To build the WASM and run all crate tests (as defined in the Makefile):

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

3. Place `registration.yaml` in `playground/config`.

4. Run the gateway:

```shell
make run
```

5. Send requests to the Flex Gateway.

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
