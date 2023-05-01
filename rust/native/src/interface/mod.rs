use crate::interface::sound::data::AudioProvider;
use crate::interface::sound::resource::{
    ResourcePath, StaticResourceProvider, StreamingAudioProvider,
};
use cpal::Stream;
use once_cell::unsync;
use once_cell::unsync::OnceCell;
use std::num::NonZeroUsize;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::thread::spawn;

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
    Exit,
}
pub struct SoundModInterfaceBuilder<
    Static: StaticResourceProvider,
    Streaming: StreamingAudioProvider,
> {
    static_provider: Static,
    streaming_provider: Streaming,
}

impl<Static: StaticResourceProvider + 'static, Streaming: StreamingAudioProvider + 'static>
    SoundModInterfaceBuilder<Static, Streaming>
{
    pub fn new(static_provider: Static, streaming_provider: Streaming) -> Self {
        Self {
            static_provider,
            streaming_provider,
        }
    }
    pub fn run(self) -> SyncSender<McToInterfaceMessage> {
        let (sender, receiver) = sync_channel(10);
        let _ = spawn(move || {
            let asdf = self.build();
            for message in receiver.iter() {
                match message {
                    McToInterfaceMessage::PrintSoundData(p) => {
                        let sound = asdf
                            .provider
                            .new_static(&p)
                            .expect("failed to create sound");
                        dbg!(sound);
                    }
                    McToInterfaceMessage::Exit => break,
                }
            }
        });
        sender
    }
    //Runs on the interface thread.
    fn build(mut self) -> SoundModInterfaceState<Static, Streaming> {
        self.static_provider.init_on_thread();
        SoundModInterfaceState {
            provider: AudioProvider::new(self.static_provider, self.streaming_provider),
        }
    }
}
struct SoundModInterfaceState<S: StaticResourceProvider, T: StreamingAudioProvider> {
    provider: AudioProvider<S, T>,
}
