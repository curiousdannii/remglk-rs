/*

Macros!
=======

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

macro_rules! stream_op {
    ($self: tt, $str_id: expr, $func: expr) => {
        {
            let str = $self.streams.get_mut($str_id)
                .ok_or(InvalidReference)?;
            $func(str)
        }
    }
}
pub(crate) use stream_op;