arch ?= x86_64
kernel := kernel/build/$(arch)/kernel.bin
rinit := rinit/build/$(arch)/rinit.bin
libcore := build/$(arch)/libcore.rlib

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

target_spec := $(arch).json

.PHONY: all clean run rinit kernel doc-kernel doc-kernel-deploy

build/rustc-nightly-src.tar.gz:
	@mkdir -p $(shell dirname $@)
	@curl https://static.rust-lang.org/dist/2016-10-04/rustc-nightly-src.tar.gz -o $@

build/libcore/lib.rs: build/rustc-nightly-src.tar.gz
	@tar -xmf build/rustc-nightly-src.tar.gz -C build/ rustc-nightly/src/libcore --transform 's~^rustc-nightly/src/~~'

$(libcore): build/libcore/lib.rs
	@mkdir -p $(shell dirname $@)
	@$(rustc) $(rust_flags) --target=$(shell realpath $(target_spec)) --out-dir=build/$(arch) --crate-type=lib $<

kernel: $(libcore)
	@make -C kernel arch=$(arch) libcore=$(shell realpath $(libcore)) target_spec=$(shell realpath $(target_spec)) kernel

rinit: $(libcore)
	@make -C rinit arch=$(arch) libcore=$(shell realpath $(libcore)) target_spec=$(shell realpath $(target_spec)) rinit

run: kernel rinit
	@qemu-system-$(arch) -kernel $(kernel) -initrd $(rinit) -serial stdio --no-reboot

debug: kernel rinit
	@qemu-system-$(arch) -d int -no-reboot -s -S -kernel $(kernel) -initrd $(rinit) -serial stdio

noreboot: kernel rinit
	@qemu-system-$(arch) -d int -no-reboot -kernel $(kernel) -initrd $(rinit) -serial stdio

gdb:
	@gdb $(kernel) -ex "target remote :1234"

clean:
	@make -C kernel arch=$(arch) libcore=$(shell realpath $(libcore)) target_spec=$(shell realpath $(target_spec)) clean
	@make -C rinit arch=$(arch) libcore=$(shell realpath $(libcore)) target_spec=$(shell realpath $(target_spec)) clean
	@rm -r build

doc-kernel:
	@rm -rf kernel/target/doc
	@cargo rustdoc --manifest-path kernel/Cargo.toml -- \
		--no-defaults \
		--passes strip-hidden \
		--passes collapse-docs \
		--passes unindent-comments \
		--passes strip-priv-imports

doc-kernel-deploy: doc-kernel
	@rsync -vraP --delete-after kernel/target/doc/ deploy@that.world:~/~docs/rux
