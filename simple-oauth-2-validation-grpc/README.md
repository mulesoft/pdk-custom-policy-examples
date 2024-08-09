# Simple OAuth 2.0 Validation Policy Example

Use the Simple OAuth 2.0 Validation Policy as an example of how to implement gRPC calls in your custom policy.

For more information about making gRPC calls, see [Performing a gRPC Call](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-grpc-request).


## Policy use case

This example is a `gRPC` based variation of the [Simple OAuth 2.0 Validation Policy Example](https://github.com/mulesoft/pdk-custom-policy-examples/simple-oauth-2-validation). 
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

1. Add a gRPC `Service` resource defining your introspection service in the `playground/config` folder.

For example, the following resource defines the Ripley 2000 service from the use case:

``` yaml
---
apiVersion: gateway.mulesoft.com/v1alpha1
kind: Service
metadata:
    name: ripley2000
spec:
    address: h2://oauth-server:4770
```

2. Run the `build` command to compile the policy:

``` shell
make build
```

3. Configure the `playground/config/api.yaml` replacing the Ripley 2000 examples with your authentication service details:

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
        name: awesome-oauth-2-validation-v1-0-impl
      config:
        tokenExtractor: "#[dw::core::Strings::substringAfter(attributes.headers['Authorization'], 'Bearer ')]"
        # If you want to use the Oauth Service mock defined in docker-compose.
        # yaml, use `http://oauth-server:8080` for `oauthService` value. If
        # you created a local mock in your host, listening at port 5001, use
        # `http://host.docker.internal:5001`
        oauthService: http://host.docker.internal:5001
        authorization: Basic dXNlcjpwYXNz
```

4. Configure a Flex Gateway instance to debug the policy by placing a registration.yaml file in `playground/config`.


5. Run the `run` command to start the Flex Gateway instance:

``` shell
make run
```

6. Send requests to Flex Gateway by using the following command as an example:

``` shell
curl http://127.0.0.1:8081 -H "Authorization: Bearer <your.oauth2.token>"
```

7. Test both valid and invalid tokens.
