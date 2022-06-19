all:
	# Kernel
	cd kernel && make run
	
	# Userspace modules
	cd modules/program && make
	
run: all
	bochs -f bochs/bochsrc.txt -q
	# qemu-system-x86_64 -cdrom os64.iso

clean:
	rm -f os64.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean
