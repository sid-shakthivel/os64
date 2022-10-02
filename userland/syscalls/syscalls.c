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

int create_window(Window *new_window)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
        mov $11, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "m"(new_window));
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
    return (Event *)result;
}

int get_current_scancode()
{
    int64_t result;
    asm volatile("mov $16, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result));
    return (int)result;
}

int lseek(int file, int ptr, int dir)
{
    uint64_t result;
    asm volatile("mov %3, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %1, %%edx \n\t\
        mov $15, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(file), "r"(ptr), "r"(dir));
    return (int)result;
}

int paint_string(char *ptr, int wid, int x, int y)
{
    int64_t result;
    asm volatile("mov %4, %%edi \n\t\
        mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %3, %%esi \n\t\
        mov $14, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "m"(ptr), "r"(wid), "r"(x), "r"(y));
    return (int)result;
}

int initalise_window_buffer(int wid)
{
    int64_t result;
    asm volatile("mov %0, %%rbx \n\t\
                 mov $17, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(wid));
    return (int)result;
}

int copy_to_buffer(int wid, uint32_t *buffer, int y_offset)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %3, %%edx \n\t\
        mov $18, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(wid), "m"(buffer), "r"(y_offset));
    return (int)result;
}

int send_message(int cpid, int pid, char *ptr)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
        mov %2, %%ecx \n\t\
        mov %3, %%edx \n\t\
        mov $20, %%eax \n\t\
        int $0x80 \n\t\
        "
                 : "=r"(result)
                 : "r"(cpid), "r"(pid), "m"(ptr));
    return (int)result;
}
