name := librinit

include ../userspace.mk

cargo:
ifeq ($(version),release)
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBALLOC) -L $(LIBSTD_UNICODE)" cargo rustc --release --target $(TARGET_SPEC) --verbose
else
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBALLOC) -L $(LIBSTD_UNICODE)" cargo rustc --target $(TARGET_SPEC) --verbose
endif
