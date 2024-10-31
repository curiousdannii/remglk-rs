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
    switch (objclass) {
        case gidisp_Class_Fileref:
            return gidispatch_get_objrock_fileref(obj);
        case gidisp_Class_Schannel:
            return gidispatch_get_objrock_schannel(obj);
        case gidisp_Class_Stream:
            return gidispatch_get_objrock_stream(obj);
        case gidisp_Class_Window:
            return gidispatch_get_objrock_window(obj);
        default:
            __builtin_unreachable();
    }
}

glkunix_argumentlist_t *glkunix_arguments_addr(void) {
    return glkunix_arguments;
}