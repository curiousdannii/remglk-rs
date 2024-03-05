/*

Emglken JS library
==================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/emglken

*/

addToLibrary({
    emglken_fileref_exists(filename_ptr, filename_len) {
        const name = UTF8ToString(filename_ptr, filename_len)
        if (name === storyfile_name) {
            return true
        }
        return false
    },

    emglken_fileref_read__deps: ['$writeBuffer'],
    emglken_fileref_read(filename_ptr, filename_len, buffer) {
        const name = UTF8ToString(filename_ptr, filename_len)
        if (name === storyfile_name) {
            writeBuffer(buffer, storyfile_data)
            return true
        }
        return false
    },

    emglken_get_glkote_event__async: true,
    emglken_get_glkote_event__deps: ['$writeBuffer'],
    emglken_get_glkote_event(buffer) {
        return Asyncify.handleAsync(async () => {
            await new Promise(resolve => { glkote_event_ready = resolve })
            writeBuffer(buffer, glkote_event_data)
        })
    },

    emglken_send_glkote_update(update_ptr, update_len) {
        const update = UTF8ToString(update_ptr, update_len)
        const obj = JSON.parse(update)
        // TODO: Store the usage of a fileref prompt request?
        GlkOte.update(obj)
    },

    $writeBuffer(buffer, data) {
        const ptr = _malloc(data.length)
        HEAP8.set(data, ptr)
        {{{ makeSetValue('buffer', 0, 'ptr', 'i32') }}}
        {{{ makeSetValue('buffer', 4, 'data.length', 'i32') }}}
    }
})