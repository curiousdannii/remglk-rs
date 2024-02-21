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

glkunix_argumentlist_t *glkunix_arguments_addr(void) {
    return glkunix_arguments;
}