run-qemu: all
	qemu-system-x86_64 -serial stdio -cdrom os64.iso

run-bochs: all
	bochs -f bochs/bochsrc.txt -q

all:
	# Userspace modules
	# docker run --rm -v /Users/siddharth/Code/rust/os64/:/code os64/toolchain bash -c "cd code/userland/program && make all"
	cd /Users/siddharth/Code/rust/os64/userland/hello-c && make 

	# Kernel
	cd /Users/siddharth/Code/rust/os64/kernel && make run

clean:
	rm -f os64.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean