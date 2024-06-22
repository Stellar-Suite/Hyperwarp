#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <stdbool.h>
#include <inttypes.h>

int* (*rust_launch_real)();

int rust_launch() {
    printf("You are meant to call this with Hyperwarp loaded into the process. (1)\n");
    return 0;
}

int main (int argc, char *argv []) {
    // we can get the args in rust through a std module. don't want to figure out how to pass C arrays into rust
    rust_launch_real = dlsym(RTLD_NEXT, "_internal_rust_launch");
    if(rust_launch_real == NULL){
        printf("You are meant to call this with Hyperwarp loaded into the process. (2)\n");
        return 0;
    }
    rust_launch_real();    
    return 0;
}