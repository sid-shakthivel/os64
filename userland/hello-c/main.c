#include <stdio.h>
#include <stdint.h>

int main()
{
    // In at&t - source, destionation
    const char *filename = "/A.TXT";
    int file_address = (int)filename;

    // Open syscall
    asm volatile("mov $0, %%rcx \n\t\
        mov %0, %%rbx \n\t\
        mov $8, %%rax \n\t\
        int $0x80 \n\t\
        "
                 :
                 : "m"(file_address));

    const char *message = "hAllo";
    int message_address = (int)message;

    // Write syscall
    asm volatile("mov $5, %%rdx \n\t\
        mov %0, %%rcx \n\t\
        mov $0, %%rbx \n\t\
        mov $10, %%rax \n\t\
        int $0x80 \n\t\
        "
                 :
                 : "m"(message_address));

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}