#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

static int count = 0;

int main()
{
    count += 1;

    printf("COUNT = %d\n", count);

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}