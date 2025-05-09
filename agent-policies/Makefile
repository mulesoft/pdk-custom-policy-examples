TARGET                	:= wasm32-wasip1
TARGET_DIR            	:= ../target/$(TARGET)/release
SUBDIRS := $(patsubst %/Makefile,%,$(wildcard *-policy/Makefile))

build:
	cargo build --release --target $(TARGET)
test:
	cargo test
clean:
	cargo clean


.PHONY: publish $(SUBDIRS)
publish: $(SUBDIRS)
$(SUBDIRS):
	$(MAKE) -C $@ TARGET_DIR=$(TARGET_DIR) publish

# Define the run command for each subdirectory
define RUN_COMMAND
run-$(1):
	@echo "Running in $(1)"
	@$(MAKE) -C $(1) TARGET_DIR=$(TARGET_DIR) run
endef

run-mcp-logging:
	@echo "Running in mcp-logging"
	@$(MAKE) -C mcp-logging TARGET_DIR=$(TARGET_DIR) run

# Iterate over each subdirectory and create a run command
$(foreach dir,$(SUBDIRS),$(eval $(call RUN_COMMAND,$(dir))))

#.PHONY: $(foreach dir,$(SUBDIRS),run-$(dir))
#

# Define the release command for each subdirectory
define RELEASE_COMMAND
release-$(1):
	@echo "Releasing in $(1)"
	@$(MAKE) -C $(1) TARGET_DIR=$(TARGET_DIR) release
endef

# Iterate over each subdirectory and create a run and release command
$(foreach dir,$(SUBDIRS),$(eval $(call RELEASE_COMMAND,$(dir))))

# Release command that runs release in all subdirectories
release: $(foreach dir,$(SUBDIRS),release-$(dir))



