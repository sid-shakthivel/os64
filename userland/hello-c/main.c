#include <stdio.h>
#include <stdint.h>

int main()
{
    // In at&t - source, destionation
    char *message = "Hello World";
    int address = (int)message;

    // asm("xchg %bx, %bx");

    // Message to write
    asm volatile("mov $11, %%rdx \n\t\
        mov %0, %%rcx \n\t\
        mov $1, %%rbx \n\t\
        mov $10, %%rax \n\t\
        int $0x80 \n\t\
        "
                 :
                 : "m"(address));

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}