TARGET              := wasm32-wasi
TARGET_DIR          := target/$(TARGET)/release
CARGO_ANYPOINT      := cargo-anypoint
DEFINITION_NAME     = $(shell anypoint-cli-v4 pdk policy-project definition get gcl-metadata-name)
DEFINITION_GCL_PATH = $(shell anypoint-cli-v4 pdk policy-project locate-gcl definition)
ASSET_VERSION       = $(shell cargo anypoint get-version)
CRATE_NAME          = $(shell cargo anypoint get-name)
OAUTH_TOKEN         = $(shell anypoint-cli-v4 pdk get-token)
SETUP_ERROR_CMD     = (echo "ERROR:\n\tMissing custom policy project setup. Please run 'make setup'\n")

ifeq ($(OS), Windows_NT)
    SHELL = powershell.exe
    .SHELLFLAGS = -NoProfile -ExecutionPolicy Bypass -Command
endif

.phony: setup
setup: login install-cargo-anypoint ## Setup all required tools to build
	cargo +nightly fetch -Z registry-auth

.phony: build
build: build-asset-files ## Build the policy definition and implementation
	@cargo build --target $(TARGET) --release
	@cp $(DEFINITION_GCL_PATH) $(TARGET_DIR)/$(CRATE_NAME)_definition.yaml
	@cargo anypoint gcl-gen -d $(DEFINITION_NAME) -w $(TARGET_DIR)/$(CRATE_NAME).wasm -o $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml

.phony: run
run: build ## Runs the policy in local flex
	@anypoint-cli-v4 pdk log -t "warn" -m "Remember to update the config values in test/config/api.yaml file for the policy configuration"
	@anypoint-cli-v4 pdk patch-gcl -f test/config/api.yaml -p "spec.policies[0].policyRef.name" -v "$(DEFINITION_NAME)-impl"
	cp $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml test/config/custom-policies/$(CRATE_NAME)_implementation.yaml
	cp $(TARGET_DIR)/$(CRATE_NAME)_definition.yaml test/config/custom-policies/$(CRATE_NAME)_definition.yaml
	-docker compose -f ./test/docker-compose.yaml down
	docker compose -f ./test/docker-compose.yaml up

.phony: publish
publish: build ## Publish a development version of the policy
	anypoint-cli-v4 pdk policy-project publish --binaryPath $(TARGET_DIR)/$(CRATE_NAME).wasm

.phony: release
release: build ## Publish a release version
	anypoint-cli-v4 pdk policy-project release --binaryPath $(TARGET_DIR)/$(CRATE_NAME).wasm

.phony: build-asset-files
build-asset-files:
	@anypoint-cli-v4 pdk policy-project build-asset-files --version $(ASSET_VERSION) --asset-id $(CRATE_NAME)
	@cargo anypoint config-gen -p -m $(DEFINITION_GCL_PATH) -o src/generated/config.rs

.phony: login
login:
	cargo login --registry anypoint $(OAUTH_TOKEN)

.phony: install-cargo-anypoint
install-cargo-anypoint:
	cargo +nightly install cargo-anypoint@1.0.0-beta.1 --registry anypoint -Z registry-auth --config .cargo/config.toml

ifneq ($(OS), Windows_NT)
all: help

.PHONY: help
help: ## Shows this help
	@echo 'Usage: make <target>'
	@echo ''
	@echo 'Available targets are:'
	@echo ''
	@grep -Eh '^\w[^:]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-6s\033[0m %s\n", $$1, $$2}' \
		| sort
endif
