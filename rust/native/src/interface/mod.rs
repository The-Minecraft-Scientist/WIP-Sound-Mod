use once_cell::sync::OnceCell;
use std::num::NonZeroUsize;

mod sound;
mod world;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SoundModNativeCfg {
    cache_size: NonZeroUsize,
}
pub(crate) static CONFIG: OnceCell<SoundModNativeCfg> = OnceCell::new();
