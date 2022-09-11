#include "syscalls.h"
#include <stdint.h>

void _exit()
{
    asm volatile("mov $0, %rax \n\t\
        int $0x80 \n\t\
        ");
}

int close(int file)
{
    int64_t result;
    asm volatile("mov %0, %%rbx \n\t\
                 mov $1, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(file));
    return (int)result;
}

int getpid()
{
    int64_t result;
    asm volatile("mov $3, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result));
    return (int)result;
}

int isatty(int file)
{
    int64_t result;
    asm volatile("mov %0, %%rbx \n\t\
                 mov $4, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(file));
    return (int)result;
}

int open(const char *name, int flags, ...)
{
    int64_t result;
    asm volatile("mov %0, %%rcx \n\t\
        mov %1, %%rbx \n\t\
        mov $7, %%rax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "m"(name), "r"(flags));
    return (int)result;
}

int write(int file, char *ptr, int len)
{
    asm volatile("xchg %bx, %bx");
    int64_t result;
    asm volatile("mov %3, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %1, %%edx \n\t\
        mov $9, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(len), "m"(ptr), "r"(file));
    return (int)result;
}

int read(int file, char *ptr, int len)
{
    int64_t result;
    asm volatile("mov %3, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %1, %%edx \n\t\
        mov $10, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(len), "m"(ptr), "r"(file));
    return (int)result;
}

int create_window(int x, int y, int width, int height)
{
    int64_t result;
    asm volatile("mov %3, %%edi \n\t\
        mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %4, %%esi \n\t\
        mov $11, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(x), "r"(y), "r"(width), "r"(height));
    return (int)result;
}

int paint_all()
{
    int64_t result;
    asm volatile("mov $12, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result));
    return (int)result;
}

Event *get_event()
{
    int64_t result;
    asm volatile("mov $13, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result));

    // asm volatile("xchg %bx, %bx");
    return (Event *)result;
}

int paint_string(char *ptr, Window *new_window)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov $14, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "m"(ptr), "m"(new_window));
    return (int)result;
}