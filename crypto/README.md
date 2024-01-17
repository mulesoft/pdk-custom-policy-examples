# Crypto Policy Template
Use the crypto policy template as an example on how to use 3rd party librarys that provide cryptography capabilities. Note that ONLY libraries that don't depend on
OS calls can be used. For example [RustCrypto](https://github.com/RustCrypto) libraries should be compatible since they are full implemented in rust.

## Test the Policy

1. This example contains an [integration test](./tests/requests.rs) to simplify its testing. Remember to add the `registration.yaml` in the `./tests/common` folder.

The test can be invoked by using the `test` command:

``` shell
make test
```

2. This example can also be started in the playground. Remember to add the `registration.yaml` in the `./payground/config` folder. The example will start by executing the following command:
``` shell
make run
```

After starting the flex you can hit the policy using curl:

```shell
curl "http://127.0.0.1:8081" -H "nonce:5c6cd51364cd25a4a25853e085ad682c899861e24f9b36934ad07503b2e70ff3fe77c8c8e38d1e00ba501d31d920b306546de0f297fbebcaffd99c4b2457f0c3996269b2ac17aec6f5c5810748f1be7a0dc20988f0a01ca61da5563e4a3f9291ba94c75912c6fa73395fd4eae3a46021e8b34bd3223b0c14d951eead7372028f04c9b7373eaee979e0bc7eaa2ad7a09cdf54c91febdb1dc0eabe35e2bc02c02a09124d45d51ed62f126e09e12dd739ed86ff578eec13b4d396c75c3bbec8a81ac3a63bfb7f20dca311bcc4fc848597dea75f0f9bc49ca6b6e6c3f1147d119b529599235045d2b2c80aa426f81494e0fe0058fd24a1766931dc07867f887c0424" -v
```

You should get the encrypted response in the payload.