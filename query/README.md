# Query Policy Template
Use the query policy template as an example on how to handle query parameters. ProxyWasm exposes the query parameters
as part of the path string. Due to this we need to leverage a third party dependency to parse, extract and modify the query parameters.
In this example this is done by using the `url` dependency which can be found in the `cargo.toml`

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
curl "http://127.0.0.1:8081?key=value&extra&absent=absent" -v
```
If everything was fine, you should see in the response that the backend received the query parameters as headers.
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