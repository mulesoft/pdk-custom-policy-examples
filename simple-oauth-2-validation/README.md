# Simple OAuth 2.0 Validation Policy Template
Use the simple OAuth 2.0 validation policy template as an example of how you create a policy that makes HTTP requests.

## Policy Use Case
An antique store that buys and sells items needs to track the stock of each item in its collection and who buys it. To track the inventory movement, the company uses an API.

Because the company is franchising, they need to expose the API on the internet for all the store managers to access. To do this, they must protect the API with an authentication layer.

For their identity management provider, they hired the “Ripley 2000” Authentication service.

Like many other identity providers, Ripley 2000 complies with OpenID standards and provides an introspection endpoint to validate OAuth 2.0 tokens.

The RFC 7662: Token Introspection extension specifies the request and response format of the introspection endpoint and states that an authentication mechanism is required. However, the RFC 7662 does not specify which authentication mechanism should be used.

The introspection endpoint provided by “Ripley 2000” requires a custom token to be sent in an authorization header with the request. The policy reads the token from the incoming request and sends it to the introspection endpoint to validate if the token is active. If any validation error occurs, the policy logs the error and rejects the incoming request.

To reuse the policy, the introspection endpoint URL, how to extract the OAuth 2.0 token, and the custom authorization values are configurable.

## Test the Policy

To test the policy:

1.  Add a `Service` resource defining your introspection service in the `test/config` folder.

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

    If you don’t have an introspection service, you can use the mock that is already defined in `test/docker-compose.yaml` named `oauth-service` which always returns a valid response to the policy by setting the address to `http://oauth-server:8080` in the above YAML.

    </div>

2.  Run the `build` command to compile the policy:

    ``` ssh
    make build
    ```
    
3. Configure the `test/config/api.yaml` replacing the Ripley 2000 examples with your authentication service details:

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
            upstream: ripley2000.default.svc
            host: ripley:5001
            path: /authorize
            authorization: Basic dXNlcjpwYXNz
    ```

3.  Configure a Flex Gateway instance to debug the policy by placing a registration.yaml file in `test/config`.


4.  Run the `run` command to start the Flex Gateway instance:

    ``` ssh
    make run
    ```

5.  Send requests to Flex Gateway by using the following command as
    example:

    ``` ssh
    curl http://127.0.0.1:8081 -H "Authorization: Bearer <your.oauth2.token>"
    ```

    Test both valid and invalid tokens.
