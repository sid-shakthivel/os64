#include <fcntl.h>
#include <stdlib.h>

extern void exit(int code);
extern int main();

extern void __libc_init_array();
extern void __libc_fini_array();

void _start()
{
    asm volatile("xchg %bx, %bx");
    int argc;
    asm volatile("mov %%rdi, %0"
                 : "=m"(argc) /* output */
                 :);

    char **argv;
    asm volatile("mov %%rsi, %0"
                 : "=m"(argv) /* output */
                 :);

    __libc_init_array();
    int ex = main(argc, argv);
    __libc_fini_array();
    exit(ex);
}
