version ?= debug
mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
root_path := $(abspath $(mkfile_path)/../..)

LD := $(GNU_PREFIX)-ld
AS := $(GNU_PREFIX)-as
C := $(GNU_PREFIX)-gcc

kernel := $(root_path)/build/$(ARCH)/libkernel.bin
rust_os := $(root_path)/target/$(TARGET)/$(version)/libkernel.a

linker_script := src/arch/$(ARCH)/linker.ld
linker_flags := -T $(linker_script)
linker_flags += -m $(LD_TARGET)
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

assembly_source_files := $(wildcard src/arch/$(ARCH)/*.S)
assembly_object_files := $(patsubst src/arch/$(ARCH)/%.S, \
	$(root_path)/build/$(ARCH)/%.o, $(assembly_source_files))

.PHONY: all build cargo

all: build

build: cargo cargo $(assembly_object_files) $(linker_script)
	@mkdir -p $(root_path)/build/$(ARCH)
	@$(LD) $(linker_flags) -o $(kernel) $(assembly_object_files) $(rust_os)

cargo:
ifeq ($(version),release)
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBCOMPILER_BUILTINS)" cargo rustc --release --target $(TARGET)
else
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBCOMPILER_BUILTINS)" cargo rustc --target $(TARGET)
endif

# compile assembly files
$(root_path)/build/$(ARCH)/%.o: src/arch/$(ARCH)/%.S
	@mkdir -p $(shell dirname $@)
	@$(C) -c $^ -o $@
