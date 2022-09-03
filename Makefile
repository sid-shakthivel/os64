KERNEL = $(shell pwd)/kernel
USERLAND_MODULE_1 = $(shell pwd)/userland/program
USERLAND_MODULE_2 = $(shell pwd)/userland/hello-1.3
USERLAND_MODULE_3 = $(shell pwd)/userland/lua
SYSCALLS = $(shell pwd)/userland/syscalls

run-qemu: all
	qemu-system-x86_64 -serial stdio -cdrom os64.iso

run-bochs: all
	bochs -f bochs/bochsrc.txt -q

all:
	# Replace filesystem
	rm -f isodir/modules/fs.img
	# cp fs.img isodir/modules

	# Compile syscalls
	cd $(SYSCALLS) && make

	# Userspace modules
	cd $(USERLAND_MODULE_1) && make 

	# cd $(USERLAND_MODULE_2) && make all

	# cd $(USERLAND_MODULE_3) && make generic

	# Kernel
	cd $(KERNEL) && make run

clean:
	rm -f os64.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean