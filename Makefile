run-qemu: all
	qemu-system-x86_64 -serial stdio -cdrom os64.iso

run-bochs: all
	bochs -f bochs/bochsrc.txt -q

all:
	# Replace filesystem
	rm -f isodir/modules/fs.img
	cp fs.img isodir/modules

	# Userspace modules
	cd /Users/siddharth/Code/rust/os64/userland/hello-c && make 

	# Kernel
	cd /Users/siddharth/Code/rust/os64/kernel && make run

clean:
	rm -f os64.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean