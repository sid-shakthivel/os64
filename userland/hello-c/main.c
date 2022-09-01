#include <stdio.h>
#include <stdint.h>
#include <string.h>

int main()
{
    // In at&t - source, destionation
    const char *filename = "/A.TXT";
    int file_address = (int)filename;

    printf("Hello World");

    // Open syscall
    // u_int64_t a;
    // asm volatile("mov $0, %%rcx \n\t\
    //     mov %0, %%rbx \n\t\
    //     mov $8, %%rax \n\t\
    //     int $0x80 \n\t\
    //     "
    //              : "=r"(a)
    //              : "m"(file_address));

    // const char *message = "hAllo";
    // int message_address = (int)message;
    // uint64_t message_length = (uint64_t)strlen(message);

    // Write syscall
    // asm volatile("mov $5, %%rdx \n\t\
    //     mov %0, %%rcx \n\t\
    //     mov %1, %%rbx \n\t\
    //     mov $10, %%rax \n\t\
    //     int $0x80 \n\t\
    //     "
    //              :
    //              : "m"(message_address), "r"(a));

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}