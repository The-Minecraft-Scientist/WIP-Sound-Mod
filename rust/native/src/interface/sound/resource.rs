





use std::fmt::{Debug, Display, Formatter, Write};









use thiserror::Error;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct ResourcePath(pub String);

impl Display for ResourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
pub trait StaticResourceProvider: Send + Sync + Debug {
    /// Attempts to load the entirety of a resource immediately
    fn oneshot(&mut self, id: &ResourcePath, buffer: &mut Vec<u8>) -> Result<(), ResourceError>;
    /// Called before a StaticResourceProvider is used on any given thread
    fn init_on_thread(&mut self);
}
pub trait StreamingAudioProvider: Send + Sync + Debug {
    /// Attempts to fill a byte buffer with new data. Returns the number of bytes written.
    /// 0 is considered a valid output, not a failure condition.
    fn read(&mut self, id: usize, buf: &mut [u8]) -> usize;
    fn init_on_thread(&mut self);
}
//Cursed dummy impl
//TODO: don't forget to remove this :/
impl StreamingAudioProvider for () {
    fn read(&mut self, _id: usize, _buf: &mut [u8]) -> usize {
        0
    }
    fn init_on_thread(&mut self) {}
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
