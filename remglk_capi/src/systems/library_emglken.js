/*

Emglken JS library
==================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/emglken

*/

addToLibrary({
    emglken_fileref_delete(fref_ptr, fref_len) {
        const fref = JSON.parse(UTF8ToString(fref_ptr, fref_len))
        Dialog.file_remove_ref(fref)
    },

    emglken_fileref_exists(fref_ptr, fref_len) {
        const fref = JSON.parse(UTF8ToString(fref_ptr, fref_len))
        if (fref.filename === storyfile_name) {
            return true
        }
        else {
            return Dialog.file_ref_exists(fref)
        }
    },

    emglken_fileref_read__deps: ['$writeBuffer'],
    emglken_fileref_read(fref_ptr, fref_len, buffer) {
        const fref = JSON.parse(UTF8ToString(fref_ptr, fref_len))
        if (fref.filename === storyfile_name) {
            writeBuffer(buffer, storyfile_data)
            return true
        }
        return false
    },

    emglken_fileref_temporary__deps: ['$FILETYPES', '$writeBufferJSON'],
    emglken_fileref_temporary(filetype, buffer) {
        let fref = Dialog.file_construct_temp_ref(FILETYPES[filetype])
        writeBufferJSON(buffer, fref)
    },

    emglken_get_glkote_event__async: true,
    emglken_get_glkote_event__deps: ['$writeBufferJSON'],
    emglken_get_glkote_event(buffer) {
        return Asyncify.handleAsync(async () => {
            await new Promise(resolve => { glkote_event_ready = resolve })
            writeBufferJSON(buffer, glkote_event_data)
        })
    },

    emglken_send_glkote_update(update_ptr, update_len) {
        const obj = JSON.parse(UTF8ToString(update_ptr, update_len))
        // TODO: Store the usage of a fileref prompt request?
        GlkOte.update(obj)
    },

    $FILETYPES: ['data', 'save', 'transcript', 'command'],

    $writeBuffer(buffer, data) {
        const ptr = _malloc(data.length)
        HEAP8.set(data, ptr)
        {{{ makeSetValue('buffer', 0, 'ptr', 'i32') }}}
        {{{ makeSetValue('buffer', 4, 'data.length', 'i32') }}}
    },

    $writeBufferJSON__deps: ['$writeBuffer'],
    $writeBufferJSON(buffer, data) {
        const json = JSON.stringify(data)
        writeBuffer(buffer, encoder.encode(json))
    },
})