CC = x86_64-elf-gcc
LINKER = x86_64-elf-ld

CFLAGS = -ffreestanding -O2 -Wall -Wextra
LINKER_FLAGS = -ffreestanding -O2 -nostdlib -z max-page-size=4096

LINKER_FILE = ../linker.ld
LIBC = $(shell pwd)/../libc/build/x86_64-sidos/lib/libc.a
LIBM = $(shell pwd)/../libc/build/x86_64-sidos/lib/libm.a
LIBG = $(shell pwd)/../libc/build/x86_64-sidos/lib/libg.a
LIB_HEADERS =$(shell pwd)/../libc/build/x86_64-sidos/include

all: main.o
	x86_64-elf-ld -T ${LINKER_FILE} -o program main.o -O2 -nostdlib -z max-page-size=4096 ${LIBC} ${LIBM} ${LIBG} && \
	rm -f ../../isodir/modules/program && \
 	mv program ../../isodir/modules 
 
main.o: main.c
	x86_64-elf-gcc -c main.c -o main.o -ffreestanding -nostdlib -O2 -Wall -Wextra -I ${LIB_HEADERS}

# https://stackoverflow.com/questions/23011367/arm-linux-gnu-gcc-fatal-error-stdio-h-no-such-file-or-directory