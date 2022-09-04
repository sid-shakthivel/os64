CC = x86_64-elf-gcc
LINKER = x86_64-elf-ld

CFLAGS = -ffreestanding -O2 -Wall -Wextra
LINKER_FLAGS = -ffreestanding -O2 -nostdlib -z max-page-size=4096 -lgcc

LINKER_FILE = ../linker.ld
LIBC = $(shell pwd)/../libc/build/x86_64-sidos/lib/libc.a
LIBM = $(shell pwd)/../libc/build/x86_64-sidos/lib/libm.a
LIBG = $(shell pwd)/../libc/build/x86_64-sidos/lib/libg.a
LIBLUA = $(shell pwd)/../lua/src/liblua.a
LIB_HEADERS = $(shell pwd)/../libc/build/x86_64-sidos/include
SYSCALLS = $(shell pwd)/../syscalls/syscalls.o

all: main.o
	$(CC) -T ${LINKER_FILE} -o program main.o -nostdlib -z max-page-size=4096 -lgcc $(LIBLUA) ${LIBM} ${LIBC} ${LIBG} $(SYSCALLS) && \
	rm -f ../../isodir/modules/program && \
 	mv program ../../isodir/modules 
 
main.o: main.c
	$(CC) -c main.c -o main.o -ffreestanding -nostdlib -Wall -Wextra -I ${LIB_HEADERS}

clean: 
	rm -f main.o