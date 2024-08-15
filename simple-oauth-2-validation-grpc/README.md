# Simple OAuth 2.0 Validation Policy Example

Use the Simple OAuth 2.0 Validation Policy as an example of how to implement gRPC calls in your custom policy.

For more information about making gRPC calls, see [Performing a gRPC Call](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-grpc-request).


## Policy use case

This example is a `gRPC` based variation of the [Simple OAuth 2.0 Validation Policy Example](https://github.com/mulesoft/pdk-custom-policy-examples/simple-oauth-2-validation) where an external authentication `gRPC` service defined in `./proto/auth.proto` is requested for each request to validate authentication token.
Read its use [case section](https://github.com/mulesoft/pdk-custom-policy-examples/simple-oauth-2-validation#policy-use-case) for more context.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. In the included integration tests demonstrate how to mock the upstream service and the introspection service by using [GripMock](https://github.com/tokopedia/gripmock) a gRPC mocking server. Depending on the token, the token introspection mock might accept or reject the incoming request token. For each case, the test asserts the expected policy behavior.

To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

2. Execute the `test` command:

### Run the Policy Locally

To manually test the policy:

1. Run the `build` command to compile the policy:

``` shell
make build
```

2. Configure the `playground/config/api.yaml` the default values with your authentication service details:

``` yaml
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
        name: simple-oauth-2-validation-grpc-v1-0-impl
      config:
        tokenExtractor: "#[attributes.queryParams.token]"
        oauthService: h2://gripmock:4770
        authorization: Basic dXNlcjpwYXNz
```

5. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

6. Modify `./playground/stub/auth.json` to change the auth stubs.

7. Run the `run` command to start the Flex Gateway instance:

``` shell
make run
```

8. Send requests to Flex Gateway by using the following command as an example, by placing the token in the URL query:

``` shell
curl -v "http://0.0.0.0:8081/hello?token=valid"
```

9. Test both `valid` and `not_valid` tokens.
