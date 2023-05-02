use crate::interface::sound::r#static::{StaticAudioProvider, StaticSound};
use crate::interface::sound::resource::{
    ResourceError, ResourcePath, StaticResourceProvider, StreamingAudioProvider,
};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

// All sounds are internally represented as 48khz mono PCM linear with 16 bits of depth. Blocks are BLOCK_LENGTH samples long. Default block length is 256.
#[derive(Debug, Clone)]
pub enum BlockProvider<
    T: StaticResourceProvider,
    U: StreamingAudioProvider,
    const BLOCK_LENGTH: usize = 256,
> {
    Static {
        cursor: Option<usize>,
        data: Rc<StaticSound<BLOCK_LENGTH>>,
    },
    Streaming {
        id: u64,
        provider: Rc<AudioProvider<T, U>>,
    },
}
impl<T: StaticResourceProvider, U: StreamingAudioProvider, const BLOCK_LENGTH: usize>
    BlockProvider<T, U, BLOCK_LENGTH>
{
    pub(crate) fn new_static(data: Rc<StaticSound<BLOCK_LENGTH>>) -> Self {
        Self::Static {
            cursor: Some(0),
            data,
        }
    }
    pub fn next_block(&mut self, buf: &mut [i16; BLOCK_LENGTH]) -> usize {
        match self {
            BlockProvider::Static { cursor, data } => {
                if let Some(c) = cursor {
                    *c += 1;
                    let index = *c * BLOCK_LENGTH;
                    if (index + BLOCK_LENGTH) == data.len() {
                        let _ = cursor.take();
                    }
                    //This won't panic because we padded the
                    buf.copy_from_slice(&data[index..(index + BLOCK_LENGTH)]);
                    BLOCK_LENGTH
                } else {
                    0
                }
            }
            BlockProvider::Streaming { id, provider } => 0,
        }
    }
}

#[derive(Debug)]
pub struct AudioProvider<
    T: StaticResourceProvider,
    U: StreamingAudioProvider,
    const BLOCK_SIZE: usize = 256,
> {
    //Cache for static sounds
    pub(crate) static_provider: RefCell<StaticAudioProvider<T, BLOCK_SIZE>>,
    pub(crate) streaming_provider: RefCell<PhantomData<U>>,
}
impl<T: StaticResourceProvider, U: StreamingAudioProvider, const BLOCK_SIZE: usize>
    AudioProvider<T, U, BLOCK_SIZE>
{
    pub fn new(static_resource_provider: T, _streaming_provider: U) -> Self {
        Self {
            static_provider: RefCell::new(StaticAudioProvider::<T, BLOCK_SIZE>::new(
                static_resource_provider,
                None,
            )),
            streaming_provider: RefCell::new(Default::default()),
        }
    }
    // If we know a sound shouldn't stream, use this method to acquire its blocks
    pub fn new_static(
        &self,
        path: &ResourcePath,
    ) -> Result<BlockProvider<T, U, BLOCK_SIZE>, ResourceError> {
        let sound = self.static_provider.borrow_mut().get_or_load_static(path)?;
        Ok(BlockProvider::<T, U, BLOCK_SIZE>::new_static(sound))
    }
}
