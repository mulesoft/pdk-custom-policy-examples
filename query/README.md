# Query Policy Example

Use the Query Policy as an example of how to handle query parameters in your custom policy. 

Because ProxyWasm exposes the query parameters as part of the path string, you must leverage a third-party dependency to parse, extract, and modify the query parameters.

In this example, the policy uses the `url` dependency found in the `cargo.toml` file.

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

1. Add the `registration.yaml` in the `./payground/config` folder

2. Execute the `run` command to begin testing:

``` shell
make run
```

3. Make requests to the Flex Gateway by using the following Curl command:

```shell
curl "http://127.0.0.1:8081?key=value&extra&absent=absent" -v
```

Flex Gateway should return a response with the query parameters as headers:

```json
{
  "args": {
    "absent": "absent",
    "removed": [
      "extra",
      "key"
    ]
  },
  "data": "",
  "files": {},
  "form": {},
  "headers": {
    "Accept": "*/*",
    "Host": "backend",
    "User-Agent": "curl/8.4.0",
    "X-Envoy-Expected-Rq-Timeout-Ms": "15000",
    "X-Envoy-Internal": "true",
    "X-Envoy-Original-Path": "/?absent=absent&removed=extra&removed=key",
    "X-Query-Extra": "",
    "X-Query-Key": "value",
    "X-Query-Missing": "Undefined"
  },
  "json": null,
  "method": "GET",
  "origin": "192.168.65.1",
  "url": "http://backend/anything/echo/?absent=absent&removed=extra&removed=key"
}
```