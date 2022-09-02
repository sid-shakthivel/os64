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
    st->st_mode = S_IFCHR;

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
    st->st_mode = S_IFCHR;
    return 0;
}
clock_t times(struct tms *buf)
{
    return -1;
}
int unlink(char *name)
{
    errno = ENOENT;
    return -1;
}
int wait(int *status)
{
    errno = ECHILD;
    return -1;
}

caddr_t sbrk(int incr)
{
    int64_t result;
    asm volatile("mov %0, %%rbx \n\t\
                 mov $8, %%rax \n\t\
                 int $0x80 \n\t\
                 "
                 : "=r"(result)
                 : "r"(incr));
    return (caddr_t)result;
}