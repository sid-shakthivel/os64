/* note these headers are all provided by newlib - you don't need to provide them */
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/fcntl.h>
#include <sys/times.h>
#include <sys/errno.h>
#include <sys/time.h>
#include <stdio.h>
#include <sys/stat.h>

void _exit()
{
    asm volatile("xchg %bx, %bx");

    asm volatile("mov $44, %rax \n\t\
        int $0x80 \n\t\
        ");
}
int close(int file)
{
    asm volatile("mov $55, %rax \n\t\
        int $0x80 \n\t\
        ");
    return 0;
}
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
    st->st_mode = S_IFCHR;
    return 0;
}
int getpid()
{
    return 1;
}
int isatty(int file)
{
    return 1;
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
int open(const char *name, int flags, ...)
{
    return -1;
}
int read(int file, char *ptr, int len)
{
    return 0;
}
caddr_t sbrk(int incr)
{
    return (caddr_t)0;
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
int write(int file, char *ptr, int len)
{
    // int address = (int)ptr;
    // asm volatile("mov %2, %%edx \n\t\
    //     mov %0, %%rcx \n\t\
    //     mov %1, %%ebx \n\t\
    //     mov $10, %%rax \n\t\
    //     int $0x80 \n\t\
    //     "
    //              :
    //              : "m"(address), "r"(file), "r"(len));

    asm volatile("xchg %bx, %bx");

    asm volatile("mov $69, %rax \n\t\
        int $0x80 \n\t\
        ");

    return 1;
}
int gettimeofday(struct timeval *p, void *restrict);

int yet_another_sussy_syscall()
{
    asm volatile("mov $90, %rax \n\t\
        int $0x80 \n\t\
        ");
    return -5;
}