name := libtest_userspace

include ../../userspace.mk

cargo:
	@RUSTFLAGS="-L $(LIBCORE) -L $(LIBALLOC)" cargo rustc --target $(TARGET_SPEC) --verbose
