arch ?= $(error arch not set)
libcore ?= $(error libcore path not set)
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

kernel := build/$(arch)/kernel.bin

rust_os := target/$(arch)/debug/librux.a

linker_script := src/arch/$(arch)/linker.ld

linker_flags := -T $(linker_script)
linker_flags += -Map build/$(arch)/map.txt
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

rust_flags :=

assembly_source_files := $(wildcard src/arch/$(arch)/*.S)
assembly_object_files := $(patsubst src/arch/$(arch)/%.S, \
	build/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run cargo kernel

all: $(kernel)

clean:
	@rm -r build
	@rm -r target

kernel: cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@$(ld) $(linker_flags) -o $(kernel).elf64 $(assembly_object_files) $(rust_os)
	@$(objcopy) $(kernel).elf64 -F elf32-i386 $(kernel)

cargo:
	@RUSTFLAGS="-L $(shell dirname $(libcore)) $(rust_flags)" $(cargo) rustc --target $(target_spec)

# compile assembly files
build/$(arch)/%.o: src/arch/$(arch)/%.S
	@mkdir -p $(shell dirname $@)
	@$(as) -o $@ $<
