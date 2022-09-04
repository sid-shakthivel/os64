#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "../lua/src/lua.h";
#include "../lua/src/lualib.h";
#include "../lua/src/lauxlib.h";

int main()
{
    lua_State *L = luaL_newstate();

    // write(1, "state\n", 6);

    // asm volatile("xchg %bx, %bx");

    luaL_openlibs(L);

    asm volatile("mov $100, %rax \n\t\
        int $0x80 \n\t\
        ");

    luaL_dostring(L, "print \"Hello Lua\"");

    lua_close(L);

    for (;;)
    {
        // body of the for loop
    }
    return 0;
}