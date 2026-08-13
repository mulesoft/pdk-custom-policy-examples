## Useful scripts

This scripts should be called from this repo's root

### Build 

Builds all policy examples:
`./.scripts/build.sh`

### Test

Runs the tests of all policy examples:
`./.scripts/test.sh`

Each example generates its own disconnected `registration.yaml` from the local Flex
image via its Makefile, so no registration file needs to be provided.

Set `PDK_TEST_FLEX_IMAGE_VERSION` (and optionally `PDK_TEST_FLEX_IMAGE_NAME`) to the
Flex version under test. An example that declares a higher minimum Flex version is
skipped, so any job that runs these tests inherits the same Flex-compatibility rules:
`PDK_TEST_FLEX_IMAGE_VERSION=1.11.0 ./.scripts/test.sh`

An example declares its minimum in `Cargo.toml`; examples without a declaration have
no minimum and always run:

```toml
[package.metadata.flex]
min-version = "1.12.0"
```