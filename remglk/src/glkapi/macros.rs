/*

Macros!
=======

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

macro_rules! str {
    ($self: tt, $str_id: expr) => {
        $self.streams.get($str_id).ok_or(InvalidReference)?
    }
}
pub(crate) use str;

macro_rules! str_mut {
    ($self: tt, $str_id: expr) => {
        $self.streams.get_mut($str_id).ok_or(InvalidReference)?
    }
}
pub(crate) use str_mut;

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

macro_rules! win {
    ($self: tt, $win_id: expr) => {
        $self.windows.get($win_id).ok_or(InvalidReference)?
    }
}
pub(crate) use win;

macro_rules! win_mut {
    ($self: tt, $win_id: expr) => {
        $self.windows.get_mut($win_id).ok_or(InvalidReference)?
    }
}
pub(crate) use win_mut;