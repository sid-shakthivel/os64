all:
	# Userspace modules
	docker run --rm -v /Users/siddharth/Code/rust/os64/:/code os64/toolchain bash -c "cd code/modules/program && make all"
	cd modules/program2 && make 

	# Kernel
	cd kernel && make run
	
run: all
	bochs -f bochs/bochsrc.txt -q
	# qemu-system-x86_64 -cdrom os64.iso

clean:
	rm -f os64.iso
	rm -f kernel.bin
	cd kernel && make clean
	cd modules/program && make clean