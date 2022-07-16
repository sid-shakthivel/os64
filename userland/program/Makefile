# LINKER_FILE = ../linker.ld
# PREFIX = ../../toolchain/prefix/bin/i686-elf
# LIBGCC = ../../toolchain/prefix/lib/gcc/i686-elf/12.1.0/libgcc.a
# CFLAGS = -ffreestanding -nostdlib -O2 -Wall -Wextra
# LINK_FLAGS = -nostdlib -O2 -L ${LIBGCC}

# all: main.o
# 	${PREFIX}-ld -o ${OUTPUT} -T ${LINKER_FILE} ${LINK_FLAGS} main.o

# main.o: main.c
# 	${PREFIX}-gcc -c main.c -o main.o ${CFLAGS}

ASSEMBLER = nasm
ASSEMBLER_FLAGS =  -f elf64

ASSEMBLY_SOURCES := $(wildcard */*.asm) $(wildcard *.asm)
ASSEMBLY_OBJECTS := $(patsubst %.asm, %.o, $(ASSEMBLY_SOURCES))

LINKER_FILE = ../linker.ld

all: $(ASSEMBLY_OBJECTS)
	ld -m elf_x86_64 -o program -T ${LINKER_FILE} ${ASSEMBLY_OBJECTS} -z max-page-size=4096
	rm -f ../../isodir/modules/program
	mv program ../../isodir/modules

$(ASSEMBLY_OBJECTS): $(ASSEMBLY_SOURCES)
	$(ASSEMBLER) $(ASSEMBLER_FLAGS) $(patsubst %.o, %.asm, $@) -o $@

clean:
	rm -f ${OUTPUT}