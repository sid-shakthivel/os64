#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

static int count = 0;

int main(int argc, char *argv[])
{
    printf("%d \n", argc);

    char **test = (char **)0xe5e000;

    printf("arg %d = %s\n", 0, test[1]);

    // for (int i = 0; i < argc; i++)
    // {
    //     printf("arg %d = %s\n", i, argv[i]);
    // }

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}