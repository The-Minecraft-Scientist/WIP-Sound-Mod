use crate::interface::sound::data::BlockProvider;
use crate::interface::sound::resource::ResourceError::NoTracksError;
use crate::interface::SoundModNativeCfg;
use lru::LruCache;
use samplerate_rs::{convert, ConverterType};
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter, Write};
use std::io::Cursor;
use std::marker::PhantomData;
use std::rc::Rc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::conv::IntoSample;
use symphonia::core::errors::Error::*;

use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use thiserror::Error;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ResourcePath(pub String);

impl Display for ResourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
pub trait StaticResourceProvider {
    /// Attempts to load the entirety of a resource immediately
    fn oneshot(&mut self, id: &ResourcePath, buffer: &mut Vec<u8>) -> Result<(), ResourceError>;
    /// Called before a StaticResourceProvider is used on any given thread
    fn init_on_thread(&mut self);
}
pub trait StreamingAudioProvider {
    /// Attempts to fill a byte buffer with new data. Returns the number of bytes written.
    /// 0 is considered a valid output, not a failure condition.
    fn read(&mut self, id: usize, buf: &mut [u8]) -> usize;
}
//Cursed dummy impl
//TODO: don't forget to remove this :/
impl StreamingAudioProvider for () {
    fn read(&mut self, _id: usize, _buf: &mut [u8]) -> usize {
        0
    }
}

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("resource \"{path}\" not found")]
    ResourceNotFound { path: ResourcePath },
    #[error("failed to decode ogg file")]
    SymphoniaError {
        #[from]
        src: symphonia::core::errors::Error,
    },
    #[error("No track found")]
    NoTracksError,
    #[error("No Data!")]
    NoDataError,
    #[error("Conversion error: {src}")]
    ConversionError {
        #[from]
        src: samplerate_rs::Error,
    },
    #[error("IO error")]
    IoError {
        #[from]
        src: std::io::Error,
    },
}
