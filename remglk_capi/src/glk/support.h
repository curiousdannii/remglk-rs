#ifndef REMGLK_RS_SUPPORT_START_H
#define REMGLK_RS_SUPPORT_START_H

#include "gi_dispa.h"
#include "glkstart.h"

extern void gidispatch_get_objrock_fileref(void *obj, gidispatch_rock_t *rock_ptr);
extern void gidispatch_get_objrock_stream(void *obj, gidispatch_rock_t *rock_ptr);
extern void gidispatch_get_objrock_window(void *obj, gidispatch_rock_t *rock_ptr);
extern void gidispatch_set_object_registry_rs(
    void (*regi)(void *obj, glui32 objclass, gidispatch_rock_t *rock_ptr),
    void (*unregi)(void *obj, glui32 objclass, gidispatch_rock_t objrock));
extern void gidispatch_set_retained_registry_rs(
    void (*regi)(void *array, glui32 len, char *typecode, gidispatch_rock_t *rock_ptr),
    void (*unregi)(void *array, glui32 len, char *typecode, gidispatch_rock_t objrock));

#endif /* REMGLK_RS_SUPPORT_START_H */