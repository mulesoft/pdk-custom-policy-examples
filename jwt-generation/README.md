# IP Filter Policy

This creates a JWT and returns it as the response of the incoming request.

To learn more about JWT generation, see [IP Filtering](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-jwt-generation).

## Policy Configuration

The policy accepts the following parameters:

- privateKey: The pem in pkcs8 format of the private key that will be used in RSA256 algorithm.
- kid: Key Identifier to add the Header of the JWT.
- jku: URL where the key can be retrieved from. Will be added to the JWT header. 

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains [integration tests](./tests/requests.rs) to simplify its testing.

To begin testing:

1. Add the `registration.yaml` in the `./tests/config` folder.

2. Execute the `test` command:

```shell
make test
```

### Playground Testing

To test the policy in the playground:

1. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

2. Execute the `run` command to start the Flex Gateway instance.

3. Send requests to test the policy. You'll see the JWT in the body of the response

```shell
curl http://localhost:8081 

```
