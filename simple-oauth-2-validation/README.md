# Simple OAuth 2.0 Validation Policy Example

Use the Simple OAuth 2.0 Validation Policy as an example of how to implement HTTP calls in your custom policy.

For more information about making HTTP calls, see [Performing an HTTP Call](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-http-request).


## Policy use case

An antique store that buys and sells items needs to track the stock of each item in its collection and who buys it. To track the inventory movement, the company uses an API.

Because the company is franchising, they need to expose the API on the internet for all the store managers to access. To do this, they must protect the API with an authentication layer.

For their identity management provider, they hired the “Ripley 2000” Authentication service.

Like many other identity providers, Ripley 2000 complies with OpenID standards and provides an introspection endpoint to validate OAuth 2.0 tokens.

The RFC 7662: Token Introspection extension specifies the request and response format of the introspection endpoint and states that an authentication mechanism is required. However, the RFC 7662 does not specify which authentication mechanism should be used.

The introspection endpoint provided by “Ripley 2000” requires a custom token to be sent in an authorization header with the request. The policy reads the token from the incoming request and sends it to the introspection endpoint to validate if the token is active. If any validation error occurs, the policy logs the error and rejects the incoming request.

To reuse the policy, the introspection endpoint URL, how to extract the OAuth 2.0 token, and the custom authorization values are configurable.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. In the included integration tests demonstrate how to mock the upstream service and the introspection service by using an HTTP MockServer. Depending on the token, the token introspection mock might accept or reject the incoming request token. For each case, the test asserts the expected policy behavior.

To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

2. Execute the `test` command:

### Run the Policy Locally

To test the policy:

1. Add a `Service` resource defining your introspection service in the `playground/config` folder.

For example, the following resource defines the Ripley 2000 service from the use case:

``` yaml
---
apiVersion: gateway.mulesoft.com/v1alpha1
kind: Service
metadata:
    name: ripley2000
spec:
    address: https://ripley2000:5001
```

If you don’t have an introspection service, you can use the mock that is already defined in `playground/docker-compose.yaml` named `oauth-service` which always returns a valid response to the policy by setting the address to `http://oauth-server:8080` in the above YAML.

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
