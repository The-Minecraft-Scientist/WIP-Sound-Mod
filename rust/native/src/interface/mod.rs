use std::num::NonZeroUsize;

pub mod sound;
pub mod world;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SoundModNativeCfg {
    cache_size: NonZeroUsize,
}
impl Default for SoundModNativeCfg {
    fn default() -> Self {
        Self {
            cache_size: NonZeroUsize::new(256).expect("unreachable code"),
        }
    }
}
pub(crate) static CONFIG: once_cell::sync::OnceCell<SoundModNativeCfg> =
    once_cell::sync::OnceCell::new();
