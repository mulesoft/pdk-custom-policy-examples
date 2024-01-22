# Cert Policy
Use this policy as an example on how read data concerning the connection certificates.

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
curl "https://localhost:8081" --cacert ./tests/resources/server.crt --cert ./tests/resources/client.pem --key ./tests/resources/client.key  -v
```

We get the response that reflects that the headers for the email and name were added:
```json
{
  "args": {}, 
  "data": "", 
  "files": {}, 
  "form": {}, 
  "headers": {
    "Accept": "*/*", 
    "Host": "backend", 
    "User-Agent": "curl/8.4.0", 
    "X-Envoy-Expected-Rq-Timeout-Ms": "15000", 
    "X-Envoy-Internal": "true", 
    "X-Envoy-Original-Path": "/", 
    "X-Peer-Email": "joker@phantomthieves.com", 
    "X-Peer-Name": "Joker"
  }, 
  "json": null, 
  "method": "GET", 
  "origin": "192.168.65.1", 
  "url": "https://backend/anything/echo/"
}
```