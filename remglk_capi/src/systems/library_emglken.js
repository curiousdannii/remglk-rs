/*

Emglken JS library
==================

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/emglken

*/

const EMGLKEN_JS = {
    $EMGLKEN_JS_CONSTANTS: {},
    $EMGLKEN_JS_CONSTANTS__deps: ['$writeBuffer', '$writeBufferJSON'],
    $EMGLKEN_JS_CONSTANTS__postset: `
        let emglken_files = {}
        const encoder = new TextEncoder()
    `,

    emglken_file_delete__async: true,
    emglken_file_delete(path_ptr, path_len) {
        return Asyncify.handleAsync(async () => {
            const path = UTF8ToString(path_ptr, path_len)
            await Dialog.delete(path)
        })
    },

    emglken_file_exists__async: true,
    emglken_file_exists(path_ptr, path_len) {
        return Asyncify.handleAsync(async () => {
            const path = UTF8ToString(path_ptr, path_len)
            return Dialog.exists(path)
        })
    },

    emglken_file_flush__async: true,
    emglken_file_flush() {
        return Asyncify.handleAsync(async () => {
            await Dialog.write(emglken_files)
            emglken_files = {}
        })
    },

    emglken_file_read__async: true,
    emglken_file_read(path_ptr, path_len, buffer) {
        return Asyncify.handleAsync(async () => {
            const path = UTF8ToString(path_ptr, path_len)
            const data = await Dialog.read(path)
            if (data) {
                writeBuffer(buffer, data)
                return true
            }
            return false
        })
    },

    emglken_file_write_buffer(path_ptr, path_len, buf_ptr, buf_len) {
        const path = UTF8ToString(path_ptr, path_len)
        const data = HEAP8.subarray(buf_ptr, buf_ptr + buf_len)
        emglken_files[path] = data
    },

    emglken_get_dirs(buffer) {
        const dirs = Dialog.get_dirs()
        writeBufferJSON(buffer, dirs)
    },

    emglken_get_glkote_event__async: true,
    emglken_get_glkote_event(buffer) {
        return Asyncify.handleAsync(async () => {
            if (!glkote_event_data) {
                await new Promise(resolve => { glkote_event_ready = resolve })
            }
            writeBufferJSON(buffer, glkote_event_data)
            glkote_event_data = null
        })
    },

    emglken_get_local_tz() {
        return new Date().getTimezoneOffset() * -60
    },

    emglken_send_glkote_update(update_ptr, update_len) {
        const obj = JSON.parse(UTF8ToString(update_ptr, update_len))
        GlkOte.update(obj)
    },

    emglken_set_storyfile_dir(path_ptr, path_len, buffer) {
        const path = UTF8ToString(path_ptr, path_len)
        const dirs = Dialog.set_storyfile_dir(path)
        writeBufferJSON(buffer, dirs)
    },

    $writeBuffer(buffer, data) {
        const ptr = _malloc(data.length)
        HEAP8.set(data, ptr)
        {{{ makeSetValue('buffer', 0, 'ptr', 'i32') }}}
        {{{ makeSetValue('buffer', 4, 'data.length', 'i32') }}}
    },

    $writeBufferJSON(buffer, data) {
        const json = JSON.stringify(data)
        writeBuffer(buffer, encoder.encode(json))
    },
}

autoAddDeps(EMGLKEN_JS, '$EMGLKEN_JS_CONSTANTS')
addToLibrary(EMGLKEN_JS)