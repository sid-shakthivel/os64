#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "../syscalls/syscalls.h"

static Window *new_window;
static int x_base = 5;
static int y_base = 20;

int main()
{
    // for (int i = 0; i < argc; i++)
    // {
    //     printf("arg %d = %s\n", i, argv[i]);
    // }
    // new_window = malloc(sizeof(Window));
    // new_window->x = 200;
    // new_window->y = 200;
    // new_window->width = 200;
    // new_window->height = 200;
    // new_window->name = "Window";

    // int wid = create_window(new_window);
    // initalise_window_buffer(wid);

    // paint_all();

    send_message(0, 1, "test");

    for (;;)
    {
    }
}
