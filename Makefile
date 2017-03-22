kernel := kernel/build/$(ARCH)/kernel.bin
rinit := rinit/build/$(ARCH)/rinit.bin
test-userspace := tests/userspace/build/$(ARCH)/test_userspace.bin

.PHONY: all clean run rinit kernel doc-kernel doc-kernel-deploy

kernel:
	@make -C kernel kernel

rinit:
	@make -C rinit build

test-userspace:
	@make -C tests/userspace build

run: kernel rinit
	@qemu-system-$(ARCH) -kernel $(kernel) -initrd $(rinit) -serial stdio --no-reboot

debug: kernel rinit
	@qemu-system-$(ARCH) -d int -no-reboot -s -S -kernel $(kernel) -initrd $(rinit) -serial stdio

noreboot: kernel rinit
	@qemu-system-$(ARCH) -d int -no-reboot -kernel $(kernel) -initrd $(rinit) -serial stdio

test: kernel test-userspace
	./tests/run.sh qemu-system-$(arch) -d int -no-reboot -vnc :1 -device isa-debug-exit -kernel $(kernel) -initrd $(test-userspace) -serial stdio

gdb:
	@gdb $(kernel) -ex "target remote :1234"

clean:
	@make -C kernel clean
	@make -C rinit clean
	@make -C tests/userspace clean

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
