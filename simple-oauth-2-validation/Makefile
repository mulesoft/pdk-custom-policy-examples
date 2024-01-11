TARGET                	:= wasm32-wasi
TARGET_DIR            	:= target/$(TARGET)/release
CARGO_ANYPOINT        	:= cargo-anypoint
DEFINITION_NAME        	= $(shell anypoint-cli-v4 pdk policy-project definition get gcl-metadata-name)
DEFINITION_NAMESPACE   	= $(shell anypoint-cli-v4 pdk policy-project definition get gcl-metadata-namespace)
DEFINITION_SRC_GCL_PATH = $(shell anypoint-cli-v4 pdk policy-project locate-gcl definition-src)
DEFINITION_GCL_PATH    	= $(shell anypoint-cli-v4 pdk policy-project locate-gcl definition)
CRATE_NAME             	= $(shell cargo anypoint get-name)
ANYPOINT_METADATA_JSON 	= $(shell cargo anypoint get-anypoint-metadata)
OAUTH_TOKEN            	= $(shell anypoint-cli-v4 pdk get-token)
SETUP_ERROR_CMD        	= (echo "ERROR:\n\tMissing custom policy project setup. Please run 'make setup'\n")

ifeq ($(OS), Windows_NT)
    SHELL = powershell.exe
    .SHELLFLAGS = -NoProfile -ExecutionPolicy Bypass -Command
endif

.PHONY: setup
setup: registry-creds login install-cargo-anypoint ## Setup all required tools to build
	cargo fetch

.PHONY: build
build: build-asset-files ## Build the policy definition and implementation
	@cargo build --target $(TARGET) --release
	@cp $(DEFINITION_GCL_PATH) $(TARGET_DIR)/$(CRATE_NAME)_definition.yaml
	@cargo anypoint gcl-gen -d $(DEFINITION_NAME) -n $(DEFINITION_NAMESPACE) -w $(TARGET_DIR)/$(CRATE_NAME).wasm -o $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml

.PHONY: run
run: build ## Run the policy in local flex
	@anypoint-cli-v4 pdk log -t "warn" -m "Remember to update the config values in playground/config/api.yaml file for the policy configuration"
	@anypoint-cli-v4 pdk patch-gcl -f playground/config/api.yaml -p "spec.policies[0].policyRef.name" -v "$(DEFINITION_NAME)-impl"
	@anypoint-cli-v4 pdk patch-gcl -f playground/config/api.yaml -p "spec.policies[0].policyRef.namespace" -v "$(DEFINITION_NAMESPACE)"
	rm -f playground/config/custom-policies/*.yaml
	cp $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml playground/config/custom-policies/$(CRATE_NAME)_implementation.yaml
	cp $(TARGET_DIR)/$(CRATE_NAME)_definition.yaml playground/config/custom-policies/$(CRATE_NAME)_definition.yaml
	-docker compose -f ./playground/docker-compose.yaml down
	docker compose -f ./playground/docker-compose.yaml up

.PHONY: test
test: build ## Run integration tests
	@cargo test -- --show-output

.PHONY: publish
publish: build ## Publish a development version of the policy
	anypoint-cli-v4 pdk policy-project publish --binary-path $(TARGET_DIR)/$(CRATE_NAME).wasm --implementation-gcl-path $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml

.PHONY: release
release: build ## Publish a release version
	anypoint-cli-v4 pdk policy-project release --binary-path $(TARGET_DIR)/$(CRATE_NAME).wasm --implementation-gcl-path $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml

.PHONY: build-asset-files
build-asset-files: $(DEFINITION_SRC_GCL_PATH)
	@anypoint-cli-v4 pdk policy-project build-asset-files --metadata '$(ANYPOINT_METADATA_JSON)'
	@cargo anypoint config-gen -p -m $(DEFINITION_SRC_GCL_PATH) -o src/generated/config.rs

.PHONY: login
login:
	@cargo login --registry anypoint $(OAUTH_TOKEN)

.PHONY: registry-creds
registry-creds:
	@git config --global credential."https://stgx.anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/e8cb2611-8681-4ec5-af55-6b5b812a86fa".username me
	@# First removing other password helpers for Anypoint context
	@git config --global --replace-all credential."https://stgx.anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/e8cb2611-8681-4ec5-af55-6b5b812a86fa".helper ""
	@# Finally adding the only password helper for Anypoint context
	@git config --global --add credential."https://stgx.anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/e8cb2611-8681-4ec5-af55-6b5b812a86fa".helper "!f() { test \"\$$1\" = get && echo \"password=\$$(anypoint-cli-v4 pdk get-token)\"; }; f"

.PHONY: install-cargo-anypoint
install-cargo-anypoint:
	cargo install cargo-anypoint@1.0.0-rc.2 --registry anypoint --config .cargo/config.toml

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
