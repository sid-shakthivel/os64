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
    errno = ENOMEM;
    return -1;
}
int fork()
{
    errno = EAGAIN;
    return -1;
}
int fstat(int file, struct stat *st)
{
    asm volatile("xchg %bx, %bx");
    if (st == NULL)
        return -1;
    st->st_mode = S_IFCHR;
    return 0;
}
int kill(int pid, int sig)
{
    errno = EINVAL;
    return -1;
}
int link(char *old, char *new)
{
    errno = EMLINK;
    return -1;
}
int lseek(int file, int ptr, int dir)
{
    return 0;
}
int stat(const char *file, struct stat *st)
{
    asm volatile("xchg %bx, %bx");
    st->st_mode = S_IFCHR;
    return 0;
}
clock_t times(struct tms *buf)
{
    asm volatile("xchg %bx, %bx");
    return -1;
}
int unlink(char *name)
{
    asm volatile("xchg %bx, %bx");
    errno = ENOENT;
    return -1;
}
int wait(int *status)
{
    asm volatile("xchg %bx, %bx");
    errno = ECHILD;
    return -1;
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
    asm volatile("mov %0, %%rbx \n\t\
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
    asm volatile("xchg %bx, %bx");
    return 0;
}

int gettimeofday(struct timeval *__p, void *__tz)
{
    asm volatile("xchg %bx, %bx");
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