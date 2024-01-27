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
                .map_err(|err| StreamError(err))
        }
    }
}
pub(crate) use stream_op;