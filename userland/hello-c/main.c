#include <stdio.h>

int main()
{
    // In at&t - source, destionation
    char *message = "Hello World";

    // asm("xchg %bx, %bx");

    // Message to write
    asm("mov $11, %%rdx \n\t\
        mov %0, %%rcx \n\t\
        mov $1, %%rbx \n\t\
        mov $4, %%rax \n\t\
        int $0x80 \n\t\
        "
        :
        : "m"(message[0])
        : "memory");

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}