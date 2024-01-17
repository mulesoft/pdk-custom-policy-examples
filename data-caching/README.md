# Data Caching Policy Example

This example shows how to use cache on a PDK policy.

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

## Integration tests

The integration tests included in the `tests` directory show the policy behavior in a scenario where it always caches the upstream response, but the amount of simultaneously cached requests is just 1. Therefore, it will always cache the latest distinct request, but with every new cached request, it forgets the previous one.
