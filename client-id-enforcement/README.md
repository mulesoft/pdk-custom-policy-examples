# Client ID Enforcement Policy Example

Uses the Client ID Enforcement Policy as an example of how to use the Contracts Validation library in your custom policy.

For more information about Contracts Validation library, see [Using Contracts Validation Library Functions](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-contracts).


## Policy use case

An API requires authentication and/or authorizaction based on based on SLA tiers.
The policy can be configured in 2 modes:

1. authentication: Validates Basic Auth credentials
2. authorization: Validates Client ID authorization extracted from Basic Auth credentials.

## Test the Policy
Run the playground with `make run` command.

To find the prereqs for using either environment and to learn more about either environment, see:

* [Debug Policies With the PDK Playground](https://docs.mulesoft.com/pdk/latest/policies-pdk-debug-local).
