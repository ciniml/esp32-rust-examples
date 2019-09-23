TARGET_ARCH := xtensa-esp32-none-elf

$(COMPONENT_LIBRARY): $(COMPONENT_PATH)/target/$(TARGET_ARCH)/release/librust_main.a

$(COMPONENT_PATH)/target/$(TARGET_ARCH)/release/librust_main.a: $(COMPONENT_PATH)/Cargo.toml $(wildcard $(COMPONENT_PATH)/src/*.rs)
	cd $(COMPONENT_PATH); PROJECT_BUILD_INCLUDE_PATH=$(PROJECT_PATH)/build/include cargo xbuild --target $(TARGET_ARCH) --release

COMPONENT_ADD_LDFLAGS := -L$(COMPONENT_PATH)/target/$(TARGET_ARCH)/release -lrust_main

COMPONENT_EXTRA_CLEAN := $(COMPONENT_PATH)/target
