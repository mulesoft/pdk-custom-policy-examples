TARGET                	:= wasm32-wasi
TARGET_DIR            	:= target/$(TARGET)/release
CARGO_ANYPOINT        	:= cargo-anypoint
POLICY_REF_NAME_SUFFIX 	:= -impl
DEFINITION_NAME        	= $(shell anypoint-cli-v4 pdk policy-project definition get gcl-metadata-name)
DEFINITION_NAMESPACE   	= $(shell anypoint-cli-v4 pdk policy-project definition get gcl-metadata-namespace)
DEFINITION_SRC_GCL_PATH = $(shell anypoint-cli-v4 pdk policy-project locate-gcl definition-src)
DEFINITION_GCL_PATH    	= $(shell anypoint-cli-v4 pdk policy-project locate-gcl definition)
CRATE_NAME             	= $(shell cargo anypoint get-name)
OAUTH_TOKEN            	= $(shell anypoint-cli-v4 pdk get-token)
POLICY_REF_NAME        	= $(DEFINITION_NAME)$(POLICY_REF_NAME_SUFFIX)
SETUP_ERROR_CMD        	= (echo "ERROR:\n\tMissing custom policy project setup. Please run 'make setup'\n")

ifeq ($(OS), Windows_NT)
    SHELL = powershell.exe
    .SHELLFLAGS = -NoProfile -ExecutionPolicy Bypass -Command
	ifneq ($(shell make -v | FIND "GNU Make 4"),)
		ANYPOINT_METADATA_JSON  = $(shell cargo anypoint get-anypoint-metadata | ConvertTo-Json)
	else
		ANYPOINT_METADATA_JSON  = $(shell cargo anypoint get-anypoint-metadata)
	endif
else
	ANYPOINT_METADATA_JSON  = $(shell cargo anypoint get-anypoint-metadata)
endif

.PHONY: setup
setup: registry-creds login install-cargo-anypoint ## Setup all required tools to build
	cargo fetch

.PHONY: build
build: build-asset-files ## Build the policy definition and implementation
	@cargo build --target $(TARGET) --release
	@cp "$(DEFINITION_GCL_PATH)" "$(TARGET_DIR)/$(CRATE_NAME)_definition.yaml"
	@cargo anypoint gcl-gen -d $(DEFINITION_NAME) -n $(DEFINITION_NAMESPACE) -w $(TARGET_DIR)/$(CRATE_NAME).wasm -o $(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml
	@echo $(POLICY_REF_NAME) > target/policy-ref-name.txt

.PHONY: run
run: build ## Run the policy in local flex
	@anypoint-cli-v4 pdk log -t "warn" -m "Remember to update the config values in playground/config/api.yaml file for the policy configuration"
	@anypoint-cli-v4 pdk patch-gcl -f playground/config/api.yaml -p "spec.policies[0].policyRef.name" -v "$(POLICY_REF_NAME)"
	@anypoint-cli-v4 pdk patch-gcl -f playground/config/api.yaml -p "spec.policies[0].policyRef.namespace" -v "$(DEFINITION_NAMESPACE)"
ifeq ($(OS), Windows_NT)
	rm -Force playground/config/custom-policies/*.yaml
else
	rm -f playground/config/custom-policies/*.yaml
endif
	cp "$(TARGET_DIR)/$(CRATE_NAME)_implementation.yaml" "playground/config/custom-policies/$(CRATE_NAME)_implementation.yaml"
	cp "$(TARGET_DIR)/$(CRATE_NAME)_definition.yaml" "playground/config/custom-policies/$(CRATE_NAME)_definition.yaml"
	-docker compose -f ./playground/docker-compose.yaml down
	docker compose -f ./playground/docker-compose.yaml up

.PHONY: test
test: build ## Run integration tests
	@cargo test -- --nocapture

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
	@git config --global credential."https://anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/19f9d123-5775-44d7-a67f-49328cfa00b0".username me
ifeq ($(OS), Windows_NT)
	@# First removing other password helpers for Anypoint context
	@git config --global --replace-all credential."https://anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/19f9d123-5775-44d7-a67f-49328cfa00b0".helper `"`"
	@# Finally adding the only password helper for Anypoint context
	@git config --global --add credential."https://anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/19f9d123-5775-44d7-a67f-49328cfa00b0".helper '!f() { test \"$$1\" = get && echo \"password=$$(anypoint-cli-v4 pdk get-token)\"; }; f'
else
	@# First removing other password helpers for Anypoint context
	@git config --global --replace-all credential."https://anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/19f9d123-5775-44d7-a67f-49328cfa00b0".helper ""
	@# Finally adding the only password helper for Anypoint context
	@git config --global --add credential."https://anypoint.mulesoft.com/git/68ef9520-24e9-4cf2-b2f5-620025690913/19f9d123-5775-44d7-a67f-49328cfa00b0".helper "!f() { test \"\$$1\" = get && echo \"password=\$$(anypoint-cli-v4 pdk get-token)\"; }; f"
endif

.PHONY: install-cargo-anypoint
install-cargo-anypoint:
	cargo install cargo-anypoint@1.2.0 --registry anypoint --config .cargo/config.toml

.PHONY: show-policy-ref-name
show-policy-ref-name:
	@echo $(POLICY_REF_NAME)

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
