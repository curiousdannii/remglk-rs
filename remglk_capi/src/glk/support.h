#ifndef REMGLK_RS_SUPPORT_START_H
#define REMGLK_RS_SUPPORT_START_H

#include "gi_dispa.h"
#include "glkstart.h"

extern gidispatch_rock_t gidispatch_get_objrock_fileref(void *obj);
extern gidispatch_rock_t gidispatch_get_objrock_stream(void *obj);
extern gidispatch_rock_t gidispatch_get_objrock_window(void *obj);

glkunix_argumentlist_t *glkunix_arguments_addr(void);

#endif /* REMGLK_RS_SUPPORT_START_H */