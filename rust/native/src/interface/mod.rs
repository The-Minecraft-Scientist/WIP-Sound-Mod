use crate::interface::sound::data::AudioProvider;
use crate::interface::sound::resource::{
    ResourcePath, StaticResourceProvider, StreamingAudioProvider,
};

use crossbeam::channel::{Receiver, Sender};
use std::num::NonZeroUsize;
use std::ops::Deref;
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
            cache_size: NonZeroUsize::new(1024).expect("unreachable code"),
        }
    }
}
pub(crate) static CONFIG: once_cell::sync::OnceCell<SoundModNativeCfg> =
    once_cell::sync::OnceCell::new();
#[derive(Debug)]
pub enum InterfaceToMcTalkBack {
    NewSound(u32),
    IsStopped(bool),
    IsPlaying(bool),
}

#[derive(Clone, Debug)]
pub enum SoundUpdateType {
    /// Permanently closes this sound
    Close,
    /// Play this sound
    Play,
    /// Pause this sound
    Pause,
    /// Resume this sound
    Resume,
    /// Respond with the playback state of this sound
    CheckIsPlaying,
    /// Respond whether this sound has been stopped
    CheckIsStopped,
    /// Set the position of this sound
    SetPosition(f64, f64, f64),
    /// Set the pitch of this sound
    SetPitch(f32),
    /// Whether this sound should restart after it completes
    SetLooping(bool),
    /// Whether the location should be relative to the listener or relative to the world origin
    SetRelative(bool),
    /// Set the static sound at this ResourcePath as this sound
    SetPath(ResourcePath),
    /// Same as above. Streaming in OGGs is not currently implemented
    SetPathStreaming(ResourcePath),
    /// Sets a custom stream implementation. All this contains is an integer id, this logic should be handled by the StreamingAudioProvider.
    SetCustomStreamImpl(u32),
}
#[derive(Debug, Clone)]
pub struct UpdateSound {
    id: u32,
    change: SoundUpdateType,
}

impl UpdateSound {
    pub fn new(id: u32, change: SoundUpdateType) -> Self {
        Self { id, change }
    }
}
///General architecture is to pass in an Arc<AtomicT> and copy into/out of it.
#[derive(Clone, Debug)]
pub enum McToInterfaceMessage {
    Change(UpdateSound),
    NewSound,
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
    pub fn run(
        self,
    ) -> (
        Sender<McToInterfaceMessage>,
        Receiver<InterfaceToMcTalkBack>,
    ) {
        use crate::interface::McToInterfaceMessage::*;
        let (from_sender, from_receiver) = crossbeam::channel::unbounded();
        let (to_sender, to_receiver) = crossbeam::channel::unbounded();
        let _ = spawn(move || {
            let state = self.build();
            let mut latest_uuid = 0u32;
            for message in from_receiver.iter() {
                match message {
                    NewSound => {
                        latest_uuid += 1;
                        to_sender
                            .send(InterfaceToMcTalkBack::NewSound(latest_uuid))
                            .expect("failed to send new sound uuid");
                    }
                    Change(UpdateSound { id: _, change }) => {
                        use crate::interface::SoundUpdateType::*;
                        match change {
                            Play => {}
                            SetPath(p) => {
                                let mut buf = [0i16; 256];
                                state.provider.new_static(&p).unwrap().next_block(&mut buf);
                            }
                            _ => {}
                        }
                    }
                    PrintSoundData(p) => {
                        let _sound = state
                            .provider
                            .new_static(&p)
                            .expect("failed to create sound");
                    }
                    Exit => break,
                }
            }
        });
        (from_sender, to_receiver)
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
