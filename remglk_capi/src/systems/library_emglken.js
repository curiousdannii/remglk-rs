/*

Emglken JS library
==================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/emglken

*/

const EMGLKEN_JS = {
    $EMGLKEN_JS_CONSTANTS: {},
    $EMGLKEN_JS_CONSTANTS__deps: ['$writeBuffer', '$writeBufferJSON'],
    $EMGLKEN_JS_CONSTANTS__postset: `
        const encoder = new TextEncoder()

        const FILETYPES = ['data', 'save', 'transcript', 'command']
        const filemode_Read = 0x02
        const filemode_Write = 0x01
        const seekmode_Start = 0
        const seekmode_End = 2
    `,

    emglken_file_delete(path_ptr, path_len) {
        const path = UTF8ToString(path_ptr, path_len)
        const fref = {filename: path}
        Dialog.file_remove_ref(fref)
    },

    emglken_file_exists(path_ptr, path_len) {
        const path = UTF8ToString(path_ptr, path_len)
        const fref = {filename: path}
        if (fref.filename === storyfile_name) {
            return true
        }
        else {
            return Dialog.file_ref_exists(fref)
        }
    },

    emglken_file_read(path_ptr, path_len, buffer) {
        const path = UTF8ToString(path_ptr, path_len)
        const fref = {filename: path}
        let data
        if (fref.filename === storyfile_name) {
            data = storyfile_data
        }
        else if (Dialog.streaming) {
            const fstream = Dialog.file_fopen(filemode_Read, fref)
            if (fstream) {
                fstream.fseek(0, seekmode_End)
                const len = fstream.ftell()
                data = new Uint8Array(len)
                fstream.fseek(0, seekmode_Start)
                fstream.fread(data)
                fstream.fclose()
            }
        }
        else {
            data = Dialog.file_read(fref)
        }
        if (data) {
            writeBuffer(buffer, data)
            return true
        }
        return false
    },

    emglken_file_write_buffer(path_ptr, path_len, buf_ptr, buf_len) {
        const path = UTF8ToString(path_ptr, path_len)
        const fref = {filename: path}
        const buffer = HEAP8.subarray(buf_ptr, buf_ptr + buf_len)
        if (Dialog.streaming) {
            const fstream = Dialog.file_fopen(filemode_Write, fref)
            if (fstream) {
                const node_buffer = fstream.BufferClass.from(buffer)
                fstream.fwrite(node_buffer)
                fstream.fclose()
            }
        }
        else {
            data = Dialog.file_write(fref, buffer)
        }
    },

    emglken_get_glkote_event__async: true,
    emglken_get_glkote_event(buffer) {
        return Asyncify.handleAsync(async () => {
            await new Promise(resolve => { glkote_event_ready = resolve })
            writeBufferJSON(buffer, glkote_event_data)
        })
    },

    emglken_send_glkote_update(update_ptr, update_len) {
        const obj = JSON.parse(UTF8ToString(update_ptr, update_len))
        GlkOte.update(obj)
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