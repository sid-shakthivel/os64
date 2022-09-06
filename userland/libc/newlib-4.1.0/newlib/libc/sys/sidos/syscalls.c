/* note these headers are all provided by newlib - you don't need to provide them */
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/fcntl.h>
#include <sys/times.h>
#include <sys/errno.h>
#include <sys/time.h>
#include <stdio.h>
#include <sys/stat.h>

#include "../../../../../../syscalls/syscalls.h"

// POSIX Syscalls (Transition to syscalls.c)

char **environ; /* pointer to array of char * strings that define the current environment variables */
int execve(char *name, char **argv, char **env)
{
    asm volatile("mov $100, %rax \n\t\
    int $0x80 \n\t\
        ");
    errno = ENOMEM;
    return -1;
}
int fork()
{
    asm volatile("mov $101, %rax \n\t\
        int $0x80 \n\t\
        ");
    errno = EAGAIN;
    return -1;
}
int fstat(int file, struct stat *st)
{
    if (st == NULL)
        return -1;
    st->st_mode = S_IFCHR;
    return 0;
}
int kill(int pid, int sig)
{
    asm volatile("mov $103, %rax \n\t\
        int $0x80 \n\t\
        ");
    errno = EINVAL;
    return -1;
}
int link(char *old, char *new)
{
    asm volatile("mov $104, %rax \n\t\
        int $0x80 \n\t\
        ");
    errno = EMLINK;
    return -1;
}
int lseek(int file, int ptr, int dir)
{
    asm volatile("mov $105, %rax \n\t\
        int $0x80 \n\t\
        ");
    return 0;
}
int stat(const char *file, struct stat *st)
{
    asm volatile("mov $106, %rax \n\t\
        int $0x80 \n\t\
        ");
    st->st_mode = S_IFCHR;
    return 0;
}
clock_t times(struct tms *buf)
{
    // asm volatile("mov $107, %rax \n\t\
    //     int $0x80 \n\t\
    //     ");
    asm volatile("xchg %bx, %bx");
    return 0;
}
int unlink(char *name)
{
    asm volatile("mov $108, %rax \n\t\
        int $0x80 \n\t\
        ");
    errno = ENOENT;
    return -1;
}
int wait(int *status)
{
    asm volatile("mov $108, %rax \n\t\
        int $0x80 \n\t\
        ");
    errno = ECHILD;
    return -1;
}

int gettimeofday(struct timeval *__p, void *__tz)
{
    __p->tv_sec = 0;
    __p->tv_usec = 0;
    return 0;
}

// liballoc

/*
    These functions lock/unlock memory data structures using a spinlock (will do eventually)
*/
int liballoc_lock()
{
    return 0;
}
int liballoc_unlock()
{
    return 0;
}

/*
    Allocates a number of pages
    Returns a pointer to memory
*/
void *liballoc_alloc(int pages)
{
    int64_t result;
    asm volatile("mov %1, %%ebx \n\t\
                 mov $8, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(pages));
    return (void *)result;
}
/*
    Frees previously allocated memory
    void* is a pointer to the allocated memory
    Returns 0 on success
*/
int liballoc_free(void *memory, int pages)
{
    write(1, "free\n", 5);
    return 0;
}

void *_malloc_r(struct _reent *r, size_t n)
{
    malloc(n);
}

void *_free_r(struct _reent *r, size_t n)
{
    free(n);
}

void *_realloc_r(struct _reent *r, size_t n)
{
    realloc(n);
}

void *_calloc_r(struct _reent *r, size_t n)
{
    realloc(n);
}