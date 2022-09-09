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

    Event *test = get_event();
    printf("%d\n", test->mouse_x);
    printf("%d\n", test->mouse_y);
    printf("%c\n", test->key_pressed);

    for (;;)
    {
        // body of the for loop
        // Event *test = get_event();
        // printf("%d\n", test->mouse_x);
    }
    return 0;
}