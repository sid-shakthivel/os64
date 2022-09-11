#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "../syscalls/syscalls.h"

static Window *new_window;

int main()
{
    // for (int i = 0; i < argc; i++)
    // {
    //     printf("arg %d = %s\n", i, argv[i]);
    // }
    new_window = malloc(sizeof(Window));
    new_window->x = 10;
    new_window->y = 10;
    new_window->width = 300;
    new_window->height = 300;
    new_window->x_final = 15;
    new_window->y_final = 35;

    create_window(10, 10, 300, 300);

    paint_all();

    char command[255];
    int count = 0;

    for (;;)
    {
        //     Get event (contains data of mouse, keyboard, etc)
        Event *event = get_event();

        // Check for keyboard event
        if (event->mask & 0b00000001)
        {
            if (count < 255)
            {
                int keycode = (int)event->key_pressed;
                // printf("%c\n", keycode);

                // Check for enter key being pressed and do command otherwise, append to string
                if (keycode == -116)
                {
                    new_window->y_final += 20; // Move onto next line
                    evaluate_command(command); // Evaluate command
                    memset(command, 0, 255);   // Empty string
                    count = 0;
                }
                else
                {
                    command[count] = event->key_pressed;
                    count++;
                    paint_string(command, new_window);
                }
            }
        }
    }
    return 0;
}

int evaluate_command(char command[255])
{
    if (strcmp(command, "hello") == 0)
    {
        // printf("Hello There User\n");
        paint_string("Hello there user", new_window);
    }
    else if (strcmp(command, "doom") == 0)
    {
        // printf("Brew is not installed\n");
        paint_string("Doom is not installed", new_window);
    }
    else
    {
        // printf("unknown command\n");
        paint_string("Unknown command", new_window);
    }
    new_window->y_final += 20;
}