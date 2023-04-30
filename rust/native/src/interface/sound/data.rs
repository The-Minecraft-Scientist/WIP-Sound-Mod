use std::cell::RefCell;
use std::marker::PhantomData;
use crate::interface::sound::resource::{StaticResourceProvider, StaticSound, StreamingAudioProvider};
use std::rc::Rc;

// All sounds are internally represented as 48khz mono with 16 bits of depth. Blocks are BLOCK_LENGTH samples long. Default block length is 256.
pub enum BlockProvider<const BLOCK_SIZE: usize = 256> {
    Static {
        cursor: usize,
        data: Rc<StaticSound>,
    },
}
impl<const SIZE: usize> BlockProvider<SIZE> {
    pub(crate) fn new_static(data: Rc<StaticSound>) -> Self {
        Self::Static { cursor: 0, data }
    }
}

#[derive(Debug)]
pub struct AudioProvider<T: StaticResourceProvider, U: StreamingAudioProvider> {
    //Cache for static sounds
    static_provider: RefCell<StaticAudioProvider<T>>,
    streaming_provider: RefCell<PhantomData<U>>,
}
impl<T: StaticResourceProvider, U: StreamingAudioProvider> AudioProvider<T, U> {
    pub fn new(static_resource_provider: T, _streaming_provider: U) -> Self {
        Self {
            static_provider: RefCell::new(StaticAudioProvider::new(static_resource_provider, None)),
            streaming_provider: RefCell::new(Default::default()),
        }
    }
    // If we know a sound shouldn't stream, use this method to acquire its blocks
    pub fn new_static(&self, path: &ResourcePath) -> Result<BlockProvider, ResourceError> {
        let sound = self.static_provider.borrow_mut().get_or_load_static(path)?;
        Ok(BlockProvider::new_static(sound))
    }
}
