name := librinit

include ../userspace.mk

cargo:
ifeq ($(version),release)
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBALLOC)" cargo rustc --release --target $(TARGET_SPEC) --verbose
else
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBALLOC)" cargo rustc --target $(TARGET_SPEC) --verbose
endif
