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
    new_window = malloc(sizeof(Window));
    new_window->x = 200;
    new_window->y = 200;
    new_window->width = 600;
    new_window->height = 400;
    new_window->name = "Terminal";

    int wid = create_window(new_window);
    initalise_window_buffer(wid);

    paint_all();

    char command[255];
    int count = 0;

    for (;;)
    {
        // Get event (contains data of mouse, keyboard, etc)
        Event *event = get_event();

        // Check for keyboard event
        if (event->mask & 0b00000001)
        {
            if (count < 255)
            {
                int keycode = (int)event->key_pressed;

                // Check for enter key being pressed and do command otherwise, append to string
                if (event->scancode == 0x1c)
                {
                    y_base += 20;              // Move onto next line
                    evaluate_command(command); // Evaluate command
                    memset(command, 0, 255);   // Empty string
                    count = 0;
                }
                else
                {
                    command[count] = event->key_pressed;
                    count++;
                    paint_string(command, 0, x_base, y_base);
                }
            }
        }
    }
}

int evaluate_command(char command[255])
{
    if (strcmp(command, "hello") == 0)
    {
        paint_string("Hello there user", 0, x_base, y_base);
    }
    else if (strcmp(command, "doom") == 0)
    {
        paint_string("Doom runs on sidos!", 0, x_base, y_base);
    }
    else
    {
        paint_string("Unknown command", 0, x_base, y_base);
    }
    y_base += 20;
}