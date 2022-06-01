ASSEMBLER = nasm

ASSEMBLER_FLAGS = -f elf64

ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, $(OBJ)/%.o, $(ASSEMBLY_SOURCES))

os64.iso: kernel.bin
	rm isodir/boot/kernel.bin
	cp kernel.bin isodir/boot
	docker run --rm -it -v /Users/siddharth/Code/rust/os64:/code os64/toolchain bash -c "cd code && grub-mkrescue -o os64.iso isodir"

kernel.bin: long_mode_init.o boot.o multiboot_header.o 
	docker run --rm -it -v /Users/siddharth/Code/rust/os64:/code os64/toolchain bash -c "cd code &&  ld -n -o kernel.bin -T src/linker.ld boot.o multiboot_header.o long_mode_init.o"

boot.o: src/boot.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) -o $@ $<

multiboot_header.o: src/multiboot_header.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) -o $@ $<

long_mode_init.o: src/long_mode_init.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) -o $@ $<

clean:
	rm kernel.bin
	rm os64.iso
	rm boot.o
	rm multiboot_header.o
	rm long_mode_init.o
