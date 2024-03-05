/*

RemGlk-rs support code
======================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

#include <stdlib.h>
#include "glk.h"
#include "support.h"

gidispatch_rock_t (*gli_register_arr)(void *array, glui32 len, char *typecode) = NULL;
void (*gli_unregister_arr)(void *array, glui32 len, char *typecode, gidispatch_rock_t objrock) = NULL;
gidispatch_rock_t (*gli_register_obj)(void *obj, glui32 objclass) = NULL;

gidispatch_rock_t gidispatch_get_objrock(void *obj, glui32 objclass) {
    gidispatch_rock_t rock;
    switch (objclass) {
        case gidisp_Class_Fileref:
            gidispatch_get_objrock_fileref(obj, &rock);
            return rock;
        case gidisp_Class_Stream:
            gidispatch_get_objrock_stream(obj, &rock);
            return rock;
        case gidisp_Class_Window:
            gidispatch_get_objrock_window(obj, &rock);
            return rock;
        default:
            __builtin_unreachable();
    }
}

#include <stdio.h>
// Because of a WASM ABI issue, we call the VM's registry functions indirectly
void gidispatch_call_array_register(void *array, glui32 len, char *typecode, gidispatch_rock_t *rock_ptr) {
    gidispatch_rock_t rock = gli_register_arr(array, len, typecode);
    printf("gidispatch_call_array_register %p %s %d %p\n", array, typecode, rock, rock_ptr);
    *rock_ptr = rock;
}
void gidispatch_call_array_unregister(void *array, glui32 len, char *typecode, gidispatch_rock_t objrock) {
    printf("gidispatch_call_array_unregister %p %s %d\n", array, typecode, objrock);
    gli_unregister_arr(array, len, typecode, objrock);
}
void gidispatch_call_object_register(void *obj, glui32 objclass, gidispatch_rock_t *rock_ptr) {
    gidispatch_rock_t rock = gli_register_obj(obj, objclass);
    *rock_ptr = rock;
}
void print_disprock(gidispatch_rock_t objrock) {
    printf("print_disprock %d\n", objrock);
}

void gidispatch_set_object_registry(
    gidispatch_rock_t (*regi)(void *obj, glui32 objclass),
    void (*unregi)(void *obj, glui32 objclass, gidispatch_rock_t objrock))
{
    gli_register_obj = regi;
    gidispatch_set_object_registry_rs(gidispatch_call_object_register, unregi);
}
void gidispatch_set_retained_registry(
    gidispatch_rock_t (*regi)(void *array, glui32 len, char *typecode),
    void (*unregi)(void *array, glui32 len, char *typecode, gidispatch_rock_t objrock))
{
    gli_register_arr = regi;
    gli_unregister_arr = unregi;
    gidispatch_set_retained_registry_rs(gidispatch_call_array_register, gidispatch_call_array_unregister);
}

glkunix_argumentlist_t *glkunix_arguments_addr(void) {
    return glkunix_arguments;
}