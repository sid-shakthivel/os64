TARGET = x86_64-unknown-none

R_COMPILER = cargo

ASSEMBLER = nasm
ASSEMBLER_FLAGS = -f elf64

OBJ := object_files
RUST_LIBRARY := target/$(TARGET)/debug/libos64.a
LINKER_FILE := src/linker.ld

ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, $(OBJ)/%.o, $(ASSEMBLY_SOURCES))

os64.iso: kernel.bin
	rm isodir/boot/kernel.bin
	cp kernel.bin isodir/boot
	grub-mkrescue /usr/lib/grub/i386-pc -o os64.iso isodir

kernel.bin: $(ASSEMBLY_OBJECTS) 
	ld -n --gc-sections -o kernel.bin -T ${LINKER_FILE} ${ASSEMBLY_OBJECTS} $(RUST_LIBRARY)

$(ASSEMBLY_OBJECTS): $(ASSEMBLY_SOURCES)
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $(patsubst $(OBJ)/%.o, %.asm, $@) -o $@

clean:
	$(R_COMPILER) clean
	rm kernel.bin
	rm os64.iso

run:
	$(R_COMPILER) build --target $(TARGET)
	rm -f kernel.bin
	docker run --rm -v /Users/siddharth/Code/rust/os64:/code os64/toolchain
	qemu-system-x86_64 -cdrom os64.iso
	# bochs -f bochsrc.txt -q
