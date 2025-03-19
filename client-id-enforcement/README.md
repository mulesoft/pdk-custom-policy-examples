# Client ID Enforcement Policy Example

Uses the Client ID Enforcement Policy as an example of how to use the Contracts Validation library in your custom policy.

For more information about Contracts Validation library, see [Using Contracts Validation Library Functions](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-contracts).


## Policy use case

An API requires authentication and/or authorizaction based on based on SLA tiers.
The policy can be configured in 2 modes:

1. authentication: Validates Basic Auth credentials
2. authorization: Validates Client ID authorization extracted from Basic Auth credentials.

## Test the Policy

Test the policy using either integration testing or the policy playground.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Writing Integration Tests](https://docs.mulesoft.com/pdk/latest/policies-pdk-integration-tests).
* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).

### Integration tests

This example contains an [integration test](./tests/requests.rs) to simplify its testing. In the included integration tests demonstrate how to mock the upstream service, the login service and the contracts service by using an HTTP MockServer. 

To begin testing:

1. Add the `registration.yaml` in the `./tests/common` folder.

2. Execute the `test` command:
