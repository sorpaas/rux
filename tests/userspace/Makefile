arch ?= $(error arch not set)
libcore ?= $(error libcore path not set)
liballoc ?= $(error liballoc path not set)
target_spec ?= $(error target spec not set)

ifeq ($(arch),x86_64)
    triple ?= x86_64-none-elf-
else
    $(error Unknown architecture $(arch))
endif

rustc ?= rustc
cargo ?= cargo
ld := $(triple)ld
as := $(triple)as
objdump := $(triple)objdump
objcopy := $(triple)objcopy

rust_flags :=

linker_script := src/arch/$(arch)/linker.ld

linker_flags := -T $(linker_script)
linker_flags += -Map build/$(arch)/map.txt
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

libtest-userspace := target/$(arch)/debug/libtest_userspace.a

assembly_source_files := $(wildcard src/arch/$(arch)/*.S)
assembly_object_files := $(patsubst src/arch/$(arch)/%.S, \
	build/$(arch)/%.o, $(assembly_source_files))

test-userspace := build/$(arch)/test_userspace.bin

.PHONY: clean cargo test-userspace

# compile assembly files
build/$(arch)/%.o: src/arch/$(arch)/%.S
	@mkdir -p $(shell dirname $@)
	@$(as) -o $@ $<

test-userspace: cargo $(libtest-userspace) $(assembly_object_files) $(linker_script)
	@mkdir -p build/$(arch)
	@$(ld) $(linker_flags) -o $(test-userspace) $(assembly_object_files) $(libtest-userspace)

cargo:
	@RUSTFLAGS="-L $(shell dirname $(libcore)) -L $(shell dirname $(liballoc)) $(rust_flags)" $(cargo) rustc --target $(target_spec)

clean:
	@rm -rf build
	@rm -rf target
