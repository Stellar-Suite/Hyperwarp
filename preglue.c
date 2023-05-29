#include <dlfcn.h>
#include <stdio.h>
#include <stdbool.h>
#include <inttypes.h>

void* get_my_odlsym_from_postglue();

void* (*odlsym_func)(void *handle, const char *symbol);

void *odlsym(void *handle, const char *symbol);

void preglue_plugin(){
    
}