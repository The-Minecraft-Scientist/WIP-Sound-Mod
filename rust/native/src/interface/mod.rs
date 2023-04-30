use crate::interface::sound::resource::{
    AudioProvider, ResourcePath, StaticResourceProvider, StreamingAudioProvider,
};
use std::num::NonZeroUsize;
use std::sync::mpsc::{channel, Receiver, Sender, sync_channel, SyncSender};
use std::thread::spawn;
use once_cell::unsync;
use once_cell::unsync::OnceCell;

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
pub enum McToInterfaceMessage {
    PrintSoundData(ResourcePath),
    Exit
}
struct SoundModInterfaceState<Static: StaticResourceProvider, Streaming: StreamingAudioProvider> {
}
impl<Static: StaticResourceProvider, Streaming: StreamingAudioProvider>
    SoundModInterfaceState<Static, Streaming>
{
    pub fn new( audio_provider: AudioProvider<Static, Streaming>) -> Self {
        Self { audio_provider}
    }
    pub fn run(self) -> Sender<McToInterfaceMessage>{
        let (sender, receiver) = channel();
        let _ = spawn(move|| {
            
        })
        sender
    }
}
