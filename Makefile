arch ?= x86_64

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

kernel := build/kernel-$(arch).bin

rust_os := target/$(arch)/debug/librux.a

linker_script := src/arch/$(arch)/linker.ld
target_spec := src/arch/$(arch)/$(arch).json

linker_flags := -T $(linker_script)
linker_flags += -Map build/arch/$(arch)/map.txt
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

rust_flags :=

assembly_source_files := $(wildcard src/arch/$(arch)/*.S)
assembly_object_files := $(patsubst src/arch/$(arch)/%.S, \
	build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run cargo

build/rustc-nightly-src.tar.gz:
	@mkdir -p $(shell dirname $@)
	@curl https://static.rust-lang.org/dist/2016-08-11/rustc-nightly-src.tar.gz -o $@

build/libcore/lib.rs: build/rustc-nightly-src.tar.gz
	@tar -xmf build/rustc-nightly-src.tar.gz -C build/ rustc-nightly/src/libcore --transform 's~^rustc-nightly/src/~~'

build/lib/$(arch)/libcore.rlib: build/libcore/lib.rs
	@mkdir -p $(shell dirname $@)
	@$(rustc) $(rust_flags) --target=$(target_spec) --out-dir=build/lib/$(arch) --crate-type=lib $<

all: $(kernel)

clean:
	@rm -r build
	@rm -r target

run: $(kernel)
	@qemu-system-x86_64 -kernel $(kernel) -serial stdio

$(kernel): cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@$(ld) $(linker_flags) -o $(kernel).elf64 $(assembly_object_files) $(rust_os)
	@$(objcopy) $(kernel).elf64 -F elf32-i386 $(kernel)

cargo: build/lib/$(arch)/libcore.rlib
	@RUSTFLAGS="-L build/lib/$(arch) $(rust_flags)" $(cargo) rustc --target $(target_spec)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.S
	@mkdir -p $(shell dirname $@)
	@$(as) -o $@ $<
