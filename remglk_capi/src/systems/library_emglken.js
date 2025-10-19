/*

Emglken JS library
==================

Copyright (c) 2025 Dannii Willis
MIT licenced
https://github.com/curiousdannii/emglken

*/

const EMGLKEN_JS = {
    $EMGLKEN_JS_CONSTANTS: {},
    $EMGLKEN_JS_CONSTANTS__deps: ['$common_buffer_transformer', '$writeBuffer', '$writeBufferJSON'],
    $EMGLKEN_JS_CONSTANTS__postset: `
        let emglken_files = {}
        const encoder = new TextEncoder()
    `,

    emglken_buffer_canon_decompose(buffer_ptr, buffer_len, initlen) {
        return common_buffer_transformer(buffer_ptr, buffer_len, initlen, str => str.normalize('NFD'))
    },

    emglken_buffer_canon_normalize(buffer_ptr, buffer_len, initlen) {
        return common_buffer_transformer(buffer_ptr, buffer_len, initlen, str => str.normalize('NFC'))
    },

    emglken_buffer_to_lower_case(buffer_ptr, buffer_len, initlen) {
        return common_buffer_transformer(buffer_ptr, buffer_len, initlen, str => str.toLowerCase())
    },

    emglken_buffer_to_title_case(buffer_ptr, buffer_len, initlen, lowerrest) {
        return common_buffer_transformer(buffer_ptr, buffer_len, initlen, utf32 => utf32.reduce((prev, ch, index) => {
            const special_cases = {
                ß: 'Ss', Ǆ: 'ǅ', ǅ: 'ǅ', ǆ: 'ǅ', Ǉ: 'ǈ', ǈ: 'ǈ', ǉ: 'ǈ', Ǌ: 'ǋ', ǋ: 'ǋ', ǌ: 'ǋ',
                Ǳ: 'ǲ', ǲ: 'ǲ', ǳ: 'ǲ', և: 'Եւ', ᾲ: 'Ὰͅ', ᾳ: 'ᾼ', ᾴ: 'Άͅ', ᾷ: 'ᾼ͂', ᾼ: 'ᾼ', ῂ: 'Ὴͅ',
                ῃ: 'ῌ', ῄ: 'Ήͅ', ῇ: 'ῌ͂', ῌ: 'ῌ', ῲ: 'Ὼͅ', ῳ: 'ῼ', ῴ: 'Ώͅ', ῷ: 'ῼ͂', ῼ: 'ῼ', ﬀ: 'Ff',
                ﬁ: 'Fi', ﬂ: 'Fl', ﬃ: 'Ffi', ﬄ: 'Ffl', ﬅ: 'St', ﬆ: 'St', ﬓ: 'Մն', ﬔ: 'Մե',
                ﬕ: 'Մի', ﬖ: 'Վն', ﬗ: 'Մխ',
            }
            const slightly_less_special_cases = ['ᾈᾉᾊᾋᾌᾍᾎᾏ', 'ᾘᾙᾚᾛᾜᾝᾞᾟ', 'ᾨᾩᾪᾫᾬᾭᾮᾯ']
            let thischar = String.fromCodePoint(ch)
            if (index === 0) {
                if (special_cases[thischar]) {
                    thischar = special_cases[thischar]
                }
                else if (ch >= 8064 && ch < 8112) {
                    thischar = slightly_less_special_cases[((ch - 8064) / 16) | 0][ch % 8]
                }
                else {
                    thischar = thischar.toUpperCase()
                }
            }
            else if (lowerrest) {
                thischar = thischar.toLowerCase()
            }
            return prev + thischar
        }, ''), 1)
    },

    emglken_buffer_to_upper_case(buffer_ptr, buffer_len, initlen) {
        return common_buffer_transformer(buffer_ptr, buffer_len, initlen, str => str.toUpperCase())
    },

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

    $common_buffer_transformer(buffer_ptr, buffer_len, initlen, func, dont_reduce) {
        const index = buffer_ptr >> 2
        const utf32 = HEAPU32.subarray(index, index + initlen)
        const data = dont_reduce ? utf32 : utf32.reduce((prev, ch) => prev + String.fromCodePoint(ch), '')
        const new_str = func(data)
        const newbuf = Uint32Array.from(new_str, ch => ch.codePointAt(0))
        const newlen = newbuf.length
        HEAPU32.set(newbuf.subarray(0, Math.min(buffer_len, newlen)), index)
        return newlen
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