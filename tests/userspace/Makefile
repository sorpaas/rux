test ?= $(error test target not set)
kernel ?= $(error kernel not set)
name := $(test)
librinit := target/$(ARCH)/debug/examples/lib$(name).a

include ../../userspace.mk

cargo:
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBALLOC)" cargo build --target $(TARGET_SPEC) --example $(name)

test: build
	../run.sh qemu-system-$(ARCH) -d int -no-reboot -vnc :1 -device isa-debug-exit -kernel $(kernel) -initrd $(rinit) -serial stdio
