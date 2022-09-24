#include <stdint.h>

typedef struct Event
{
    int mouse_x;
    int mouse_y;
    int scancode;
    int mask;
    char key_pressed;
} Event;

typedef struct Windowss
{
    int x;
    int y;
    int width;
    int height;
    char *name;
} Window;

void _exit();
int close(int file);
// char **environ; /* pointer to array of char * strings that define the current environment variables */
// int execve(char *name, char **argv, char **env);
// int fork();
// int fstat(int file, struct stat *st);
int getpid();
int isatty(int file);
// int kill(int pid, int sig);
// int link(char *old, char *new);
int open(const char *name, int flags, ...);
int read(int file, char *ptr, int len);
// int stat(const char *file, struct stat *st);
// clock_t times(struct tms *buf);
// int unlink(char *name);
int create_window(Window *new_window);
int paint_all();
Event *get_event();
int get_current_scancode();
int paint_string(char *ptr, int wid, int x, int y);
int initalise_window_buffer(int wid);
int copy_to_buffer(int wid, uint32_t *buffer);

// int wait(int *status);
int lseek(int file, int ptr, int dir);
int write(int file, char *ptr, int len);
// int gettimeofday(struct timeval *p, void *restrict);