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
