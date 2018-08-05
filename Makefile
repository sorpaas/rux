mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
root_path := $(abspath $(mkfile_path)/..)

kernel := $(root_path)/build/$(ARCH)/libkernel.bin

.PHONY: kernel kernel-release run clean

kernel:
	@make -C kernel build

kernel-release:
	@make -C kernel version=release build

run: kernel
	@qemu-system-$(ARCH) -d in_asm -machine virt -nographic -kernel $(kernel) --no-reboot

clean:
	@rm -rf build
	@rm -rf target
