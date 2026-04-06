# Spike Control Policy

Use the Spike Control Policy as an example of how to limit how many requests reach the backend in a time window, with optional delays and retries when the limit is reached.

This policy uses Flex Gateway spike control. For example, if you set `requests` to `100` and have two workers, up to 200 requests can be serviced during the window (per worker behavior follows the runtime).

The policy takes the following parameters:

* `requests`: The number of requests that can reach the backend in the given time window.
* `millis`: The duration in milliseconds of the time window.
* `maxAttempts`: The maximum number of throttling attempts before the request is rejected.
* `delay`: The delay in milliseconds between each throttled attempt.

To learn more about spike control, see [Configuring Spike Control](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-spike-control).

## Test the Policy

Use **unit tests** (`pdk-unit`) or the **playground**. This crate stays on `edition = "2018"` without Cargo’s resolver v2; the dev-dependency **`pdk-test`** pulls **Tokio with `full`**, which under the default resolver is unified with the **wasm32-wasip1** policy build and breaks compilation (Tokio only allows a subset of features on WASM). So **`pdk-test` is not declared here**. For Docker-based integration tests with `pdk_test`, follow the same layout as other examples (e.g. [spike](../spike)) in a setup that avoids that conflict, or use the docs:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Unit tests

```shell
make test
```

(`make test` still runs `cargo build` for WASM, then `cargo test` for the tests in `src/lib.rs`.)

### Playground Testing

To use the policy in the playground:

1. Add the `registration.yaml` in the `./playground/config` folder.

2. Execute the `run` command to begin testing:

```shell
make run
```

3. Make requests to the Flex Gateway, for example:

```shell
curl "http://localhost:8081/anything/echo/get"
```

When you exceed the configured limit within the window, the gateway responds with HTTP 429 (unless a queued attempt succeeds within `maxAttempts`).
