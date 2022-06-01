ASSEMBLER = nasm

ASSEMBLER_FLAGS = -f elf64

ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, $(OBJ)/%.o, $(ASSEMBLY_SOURCES))

os64.iso: kernel.bin
	docker run --rm -it -v /Users/siddharth/Code/rust/os64:/code os64/toolchain

kernel.bin: boot.o multiboot_header.o
	docker run --rm -it -v /Users/siddharth/Code/rust/os64:/code os64/toolchain bash -c "cd code &&  ld -n -o kernel.bin -T src/linker.ld boot.o multiboot_header.o"

boot.o: src/boot.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) -o $@ $<

multiboot_header.o: src/multiboot_header.asm
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) -o $@ $<

clean:
	rm kernel.bin
	rm os64.iso
