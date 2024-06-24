# Crypto Policy Example

Use the Crypto Policy as an example of how to use third-party party libraries that provide cryptography capabilities.

PDK does not support libraries that make OS calls. For example, the [RustCrypto](https://github.com/RustCrypto) libraries are compatible because they are fully implemented in Rust.

For more information about third-party library support, see [Use Third-Party Libraries](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-libraries).

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

To use the policy in the playground:

1. Add the `registration.yaml` in the `./playground/config` folder

2. Execute the `run` command to begin testing:

``` shell
make run
```

3. Make requests to the Flex Gateway by using the following Curl command:

```shell
curl "http://127.0.0.1:8081" -H "nonce:5c6cd51364cd25a4a25853e085ad682c899861e24f9b36934ad07503b2e70ff3fe77c8c8e38d1e00ba501d31d920b306546de0f297fbebcaffd99c4b2457f0c3996269b2ac17aec6f5c5810748f1be7a0dc20988f0a01ca61da5563e4a3f9291ba94c75912c6fa73395fd4eae3a46021e8b34bd3223b0c14d951eead7372028f04c9b7373eaee979e0bc7eaa2ad7a09cdf54c91febdb1dc0eabe35e2bc02c02a09124d45d51ed62f126e09e12dd739ed86ff578eec13b4d396c75c3bbec8a81ac3a63bfb7f20dca311bcc4fc848597dea75f0f9bc49ca6b6e6c3f1147d119b529599235045d2b2c80aa426f81494e0fe0058fd24a1766931dc07867f887c0424" -v
```

Flex Gateway returns the encrypted response in the payload.