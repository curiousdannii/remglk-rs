/*

Glk Sound Channels
==================

Copyright (c) 2024 Dannii Willis
MIT licenced
https://github.com/curiousdannii/remglk-rs

*/

use super::*;

pub const SCHANNEL_MAX_VOL: f64 = 65536.0;

pub type SoundChannelRef = GlkObject<SoundChannel>;

#[derive(Default)]
pub struct SoundChannel {
    pub ops: Vec<protocol::SoundChannelOperation>,
}

impl GlkObjectClass for SoundChannel {
    fn get_object_class_id() -> u32 {
        3
    }
}