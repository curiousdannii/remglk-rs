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

    emglken_fileref_read(filename_ptr, filename_len, buffer) {
        const name = UTF8ToString(filename_ptr, filename_len)
        if (name === storyfile_name) {
            const ptr = _malloc(storyfile_data.length)
            HEAP8.set(storyfile_data, ptr)
            {{{ makeSetValue('buffer', 0, 'ptr', 'i32') }}}
            {{{ makeSetValue('buffer', 4, 'storyfile_data.length', 'i32') }}}
            return true
        }
        return false
    },
})