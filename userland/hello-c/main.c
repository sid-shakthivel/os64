#include <stdio.h>
#include <stdint.h>
#include <string.h>

int main()
{
    // asm volatile("xchg %bx, %bx");
    printf("Yello World\n");

    fprintf(stdout, "Hello\n");
    puts("Hello World");

    // In at&t - source, destionation
    // const char *filename = "/A.TXT";
    // int file_address = (int)filename;

    // int length = 78;
    // int file = 1;

    // const char *message = "hAllo";
    // int message_address = (int)message;

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}
