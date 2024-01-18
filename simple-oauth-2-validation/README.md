# Simple OAuth 2.0 Validation Policy Example

This example shows a policy that has to perform http requests to external services, in this case, a token introspection service.

## Policy use case

An antique store that buys and sells items needs to track the stock of each item in its collection and who buys it. To track the inventory movement, the company uses an API.

Because the company is franchising, they need to expose the API on the internet for all the store managers to access. To do this, they must protect the API with an authentication layer.

For their identity management provider, they hired the “Ripley 2000” Authentication service.

Like many other identity providers, Ripley 2000 complies with OpenID standards and provides an introspection endpoint to validate OAuth 2.0 tokens.

The RFC 7662: Token Introspection extension specifies the request and response format of the introspection endpoint and states that an authentication mechanism is required. However, the RFC 7662 does not specify which authentication mechanism should be used.

The introspection endpoint provided by “Ripley 2000” requires a custom token to be sent in an authorization header with the request. The policy reads the token from the incoming request and sends it to the introspection endpoint to validate if the token is active. If any validation error occurs, the policy logs the error and rejects the incoming request.

To reuse the policy, the introspection endpoint URL, how to extract the OAuth 2.0 token, and the custom authorization values are configurable.

## Integration tests

The integration tests included in the `tests` directory show how to mock not only the upstream service, but also the introspection service, using an HTTP MockServer. Depending on the token, the token introspection mock might accept or reject the incoming request token. For each case, the test asserts the expected policy behavior.

## Run the Policy Locally

To test the policy:

1.  Add a `Service` resource defining your introspection service in the `playground/config` folder.

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

    <div class="note">

    If you don’t have an introspection service, you can use the mock that is already defined in `playground/docker-compose.yaml` named `oauth-service` which always returns a valid response to the policy by setting the address to `http://oauth-server:8080` in the above YAML.

    </div>

2.  Run the `build` command to compile the policy:

    ``` ssh
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

4.  Configure a Flex Gateway instance to debug the policy by placing a registration.yaml file in `playground/config`.


5.  Run the `run` command to start the Flex Gateway instance:

    ``` ssh
    make run
    ```

6.  Send requests to Flex Gateway by using the following command as
    example:

    ``` ssh
    curl http://127.0.0.1:8081 -H "Authorization: Bearer <your.oauth2.token>"
    ```

Test both valid and invalid tokens.
