# JWT Validation Policy Example

This example showcases the helpers provided by the PDK JWT library to extract, parse, and validate JWT tokens.

To learn more about the PDK JWT library, see [Configuring JWT Library Functions](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-jwt).

## Policy use case

A local library in a small town uses an open source software to manage their book inventory. The software provides an API to keep track of information about the book and if the books are checked out. The software contains an authentication system that leverages JSON Web Tokens (JWT).

JWTs are an industry-standard method to represent claims securely between different parties. JWTs can transport small sets of data securely. They can be digitally signed either with a secret using the HMAC algorithm or with a set of private and public keys using the RSA or ECDSA algorithms.

The library also implements a benefit system for their regular customers that return their books on time and in good condition. The benefit system uses a different open source software from the software managing the book inventory. Using the signing keys provided by the book inventory service to sign the JWT tokens, a policy can provide an authentication mechanism that reuses these tokens for the benefit service.

The policy is responsible for validating the signature of the tokens, ensuring they are not expired, and obtaining the role of the user (customer or administrator). As this information is contained in the tokens, the policy must ensure the token is current and trustworthy, and then extract the required information to forward it to the benefits service.

## Policy behavior

The policy performs several validations:

- Extracts the token
- Validates the signature and extracts the payload
- Validates the token is not expired
- Validates through dataweave one of the custom claims contained in the JWT payload
- Forwards the "username" JWT custom claim value to the upstream service in "username" header.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration Testing

This example contains an [integration test](./tests/requests.rs) to simplify its testing. To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

2. Execute the `test` command:

``` shell
make test
```

### Playground Testing

The API in `playground/config/api.yaml` file has a secret configured. If you don't change it, you can use the secrets in `/tests/resources` text files.

To use the policy in the playground:

1. Add the `registration.yaml` in the `./playground/config` folder

2. Execute the `run` command to begin testing:

3. Copy one of the secrets from the `/tests/resources` text files. Do not add a new line character to the end of the secret.

4. Make requests to the Flex Gateway by using the following Curl command:

```sh
curl --location --request GET 'localhost:8081' \
--header 'Authorization: Bearer <copied-secret>'
```

5. Additionally, you can create your own HMAC tokens by using another secret and setting custom JWT claims. There are several sites online that instantly create tokens by prompting the algorithm with the signing keys or secret and the desired JWT claims.
