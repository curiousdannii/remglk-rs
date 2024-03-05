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

    emglken_fileref_read(fref_ptr, fref_len, buffer) {
        const fref = JSON.parse(UTF8ToString(fref_ptr, fref_len))
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

    emglken_fileref_temporary(filetype, buffer) {
        let fref = Dialog.file_construct_temp_ref(FILETYPES[filetype])
        writeBufferJSON(buffer, fref)
    },

    emglken_fileref_write_buffer(fref_ptr, fref_len, buf_ptr, buf_len) {
        const fref = JSON.parse(UTF8ToString(fref_ptr, fref_len))
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