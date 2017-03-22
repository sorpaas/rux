kernel := build/$(ARCH)/kernel.bin
rust_os := target/$(ARCH)/debug/libkernel.a

linker_script := src/arch/$(ARCH)/linker.ld
linker_flags := -T $(linker_script)
linker_flags += -Map build/$(arch)/map.txt
linker_flags += --gc-sections
linker_flags += -z max-page-size=0x1000

assembly_source_files := $(wildcard src/arch/$(ARCH)/*.S)
assembly_object_files := $(patsubst src/arch/$(ARCH)/%.S, \
	build/$(ARCH)/%.o, $(assembly_source_files))

.PHONY: all clean run cargo kernel

all: $(kernel)

clean:
	@rm -r build
	@rm -r target

build: cargo $(rust_os) $(assembly_object_files) $(linker_script)
	@$(LD) $(linker_flags) -o $(kernel).elf64 $(assembly_object_files) $(rust_os)
	@$(OBJCOPY) $(kernel).elf64 -F elf32-i386 $(kernel)

cargo:
	@RUSTFLAGS="-L $(LIBCORE)" cargo rustc --target $(TARGET_SPEC)

# compile assembly files
build/$(arch)/%.o: src/arch/$(ARCH)/%.S
	@mkdir -p $(shell dirname $@)
	@$(AS) -o $@ $<
