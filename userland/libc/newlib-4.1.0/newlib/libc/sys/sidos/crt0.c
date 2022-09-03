#include <fcntl.h>
#include <stdlib.h>

extern void exit(int code);
extern int main();

extern void __libc_init_array();
extern void __libc_fini_array();

void _start()
{
    __libc_init_array();
    int ex = main();
    __libc_fini_array();
    exit(ex);
}
