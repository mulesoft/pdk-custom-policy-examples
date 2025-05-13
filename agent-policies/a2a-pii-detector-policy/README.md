# A2A PII Detector Policy Example

This policy detects sensitive information in prompts sent to and from an agent. The policy checks for US Social Security numbers, email addresses, credit card numbers, and US phone numbers. If sensitive information is found, the prompt is either rejected or logged, depending on the selected action.

## Testing the Policy

Test the policy by publishing it to Exchange.

### Setting Up the Example Policy

Follow [Set Up an Example Policy Project](https://docs.mulesoft.com/pdk/latest/policies-pdk-policy-templates#set-up-an-example-policy-project).

### Uploading Custom Policy to Exchange

Follow [Uploading Custom Policies to Exchange](https://docs.mulesoft.com/pdk/latest/policies-pdk-publish-policies).

### Applying Custom Policy to an API

Follow [Applying Custom Policies](https://docs.mulesoft.com/pdk/latest/policies-pdk-apply-policies).
