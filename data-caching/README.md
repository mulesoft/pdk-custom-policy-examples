# Data Caching Policy Example

This example demonstrates how to use caching in your custom policy.

To learn more about caching, see [Sharing Data Between Workers and Caching](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-caching).

## Policy use case

An antique store that buys and sells items needs to track the stock of each item in its catalog and who buys it. To track the inventory movement, the company uses an API where each unique path represents a different item.

Because the store’s catalog items in stock change constantly during business hours but are frozen outside of business hours, the server manager decides to implement a caching policy that caches the stock during non-business hours.

The server manager implements the caching policy as follows:

1. The policy first checks the time of each incoming request.
2. If the request is outside of business hours and if a response for the path has not been previously cached, the policy caches the response.
3. The policy returns the cached response for all following requests to the path outside of business hours.
4. When business hours resume, the policy clears the cache and is ready to cache new responses during the next period of non-business hours.

To reuse the policy, the amount of cached requests and the non-business hours are configurable.

An error in the caching flow should not make a request fail. By default, the caching policy does not block requests.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. In the included integration tests, the policy always caches all requests but the max cache entries parameter is one. Therefore with every new request, it caches the response and forgets the previous cache entry.

To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

2. Execute the `test` command:

### Playground Testing

To test the policy:

1. Run the `build` command to compile the policy:

``` shell
make build
```

2. Configure the `playground/config/api.yaml` as follows:

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
        name: awesome-caching-v1-0-impl
    config:
        max_cached_values: 10
        start_hour: 18
        end_hour: 10
```

3. Configure a Flex Gateway instance to debug the policy by placing a `registration.yaml` file in `playground/config`.

4. Run the `run` command to start the Flex Gateway instance:

``` shell
make run
```

5. Send requests to Flex Gateway:

``` shell
curl http://127.0.0.1:8081/catalog/1 -H "cache_check: cache_value"
```

The upstream service used in the debugging environment included with PDK responds with an echo of the request.

6. Send another request to Flex Gateway by changing the included header:

``` shell
curl http://127.0.0.1:8081/catalog/1 -H "cache_check: cache_value1"
```

If requests are made outside of business hours, the header `cache_check: cache_value` is included in the response body instead of `cache_check: cache_value1`. If requests are made inside of business hours, the header `cache_check: cache_value1` is included in the response body.

7. Change the non-business hours and request path to view different responses.
