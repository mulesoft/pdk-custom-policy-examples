# XML Validation Policy

This policy validates incoming request bodies as well-formed XML and can enforce optional structural limits (depth, attributes, children, text length, and so on) using the PDK XML validator library on the request body stream.

To learn more about building policies with the PDK, see the [PDK documentation](https://docs.mulesoft.com/pdk/latest/).

XML Validation policy implementation:

1. The policy intercepts each incoming request. If there is **no body** (for example a `GET`), validation is skipped and the request proceeds to the upstream service.
2. If a body is present, the policy reads the **streaming** payload and validates it as XML.
3. If validation succeeds, the request proceeds to the upstream service.
4. If the document is malformed or violates a configured limit, the policy responds with **HTTP 400**.

## Policy Configuration

The policy accepts the following parameters. All are **optional**; omit a property, use `null`, or use **`0`** to mean *no limit* for that constraint. If every limit is disabled, only **malformed** XML is rejected.

- `maxDepth` (optional): Maximum element nesting depth.
- `maxAttributeCount` (optional): Maximum number of attributes on a single element.
- `maxChildCount` (optional): Maximum number of direct child elements under one parent.
- `maxTextLength` (optional): Maximum length of text content inside an element.
- `maxAttributeLength` (optional): Maximum length of an attribute value.
- `maxCommentLength` (optional): Maximum length of an XML comment.

## Test the Policy

Test the policy using unit tests or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Unit tests

This example contains unit tests in `src/lib.rs`.

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
        name: xml-validation-v1-0-impl
        namespace: default
      config:
        maxDepth: 32
        maxAttributeCount: 32
        maxChildCount: 256
        maxTextLength: 65536
        maxAttributeLength: 2048
        maxCommentLength: 4096
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

```shell
make run
```

5. Send requests to test the XML validation:

```shell
# POST with valid XML should succeed
curl -X POST "http://localhost:8081/" \
  -H "Content-Type: application/xml" \
  -d '<note><to>you</to><body>hello</body></note>'

# POST with malformed XML should return 400
curl -X POST "http://localhost:8081/" \
  -H "Content-Type: application/xml" \
  -d '<note><to></note>'

# GET without body should succeed; validation is skipped
curl "http://localhost:8081/"
```
