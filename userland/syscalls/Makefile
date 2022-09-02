CC = x86_64-elf-gcc

CFLAGS = -ffreestanding -O2 -Wall -Wextra
 
syscalls.o: syscalls.c
	$(CC) -c syscalls.c -o syscalls.o -ffreestanding -nostdlib -O0 -Wall -Wextra

clean:
	rm syscalls.o