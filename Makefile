ASSEMBLER = nasm

ASSEMBLER_FLAGS = -f elf64

OBJ := object_files

ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, $(OBJ)/%.o, $(ASSEMBLY_SOURCES))

os64.iso: kernel.bin
	rm isodir/boot/kernel.bin
	cp kernel.bin isodir/boot
	grub-mkrescue -o os64.iso isodir

kernel.bin: $(ASSEMBLY_OBJECTS)
	ld -n -o kernel.bin -T src/linker.ld ${ASSEMBLY_OBJECTS}

$(ASSEMBLY_OBJECTS): $(ASSEMBLY_SOURCES)
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $(patsubst $(OBJ)/%.o, %.asm, $@) -o $@

clean:
	rm kernel.bin
	rm os64.iso

run:
	docker run --rm -v /Users/siddharth/Code/rust/os64:/code os64/toolchain
	qemu-system-x86_64 -cdrom os64.iso
