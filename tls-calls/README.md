# TLS Calls Policy

Use the tls-calls Policy as an example of how to make https calls in your policy without installing certificates on the OS running flex.

To accomplish this you'll need to:
1. Manually configure the service with its tls policy binding. Check the ./playground/config/service.yaml.
2. Provide to the policy the service coordinates, as policy configuration or hardcoding them. Check ./playground/config/api.yaml.
3. Instantiate a service in your policy with said coordinates. Check ./src/lib.rs.

## Test the Policy

Test the policy using the playground.

To find the prereqs for using the playground environment, see:

* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Playground Testing

To use the policy in the playground:

1. Add the `registration.yaml` in the `./playground/config` folder

2. Execute the `run` command to begin testing:

``` shell
make run
```

3. Make requests to the Flex Gateway by using the following Curl command:

```shell
curl "http://localhost:8081" -v
```
Flex Gateway should return the response of the request made by the policy:

```json
{
  "args": {},
  "data": "",
  "files": {},
  "form": {},
  "headers": {
    "Connection": "close",
    "Host": "backend",
    "User-Agent": "PDK-HttpClient/1.1.0",
    "X-Envoy-Expected-Rq-Timeout-Ms": "10000",
    "X-Envoy-Internal": "true",
    "X-Forwarded-Host": "proxy"
  },
  "json": null,
  "method": "GET",
  "origin": "192.168.32.3",
  "url": "https://proxy/anything/echo/"
}
```