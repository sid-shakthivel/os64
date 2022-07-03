ASSEMBLER = nasm
ASSEMBLER_FLAGS =  -f elf64

ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, %.o, $(ASSEMBLY_SOURCES))

LINKER_FILE = ../linker.ld
OUPTUT = program

all: $(ASSEMBLY_OBJECTS)
	ld -m elf_x86_64 -o program -T ${LINKER_FILE} ${ASSEMBLY_OBJECTS} -z max-page-size=4096
	rm -f ../../isodir/modules/program
	mv program ../../isodir/modules

$(ASSEMBLY_OBJECTS): $(ASSEMBLY_SOURCES)
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $(patsubst %.o, %.asm, $@) -o $@

clean:
	rm -f ${OUTPUT}