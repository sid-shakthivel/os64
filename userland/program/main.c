#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "../syscalls/syscalls.h"

int main(int argc, char **argv)
{
    create_window(10, 10, 300, 300);

    paint_all();

    // for (int i = 0; i < argc; i++)
    // {
    //     printf("arg %d = %s\n", i, argv[i]);
    // }

    for (;;)
    {
        // body of the for loop
        char key_pressed = get_event();
        int test = (int)key_pressed;
        if (test != 49 && test != 48 && test != 0)
        {
            printf("KEY PRESSED = %c %d\n", key_pressed, test);
        }
    }
    return 0;
}