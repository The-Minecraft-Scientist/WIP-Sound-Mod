use crate::interface::sound::data::BlockProvider;
use crate::interface::sound::resource::ResourceError::NoTracksError;
use crate::interface::SoundModNativeCfg;
use lru::LruCache;
use samplerate_rs::{convert, ConverterType};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
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
/// Mono sound. 48 khz sample rate, 1 channel, 16 bit depth.
#[derive(Clone, Debug)]
pub struct StaticSound(Vec<i16>);
impl StaticSound {
    fn new(source: &mut Vec<u8>) -> Result<Self, ResourceError> {
        // Icky clone. This is the cost of dynamic dispatch ig.
        let mss =
            MediaSourceStream::new(Box::new(Cursor::new(source.to_owned())), Default::default());
        let mut fmt = symphonia::default::get_probe()
            .format(
                &Hint::with_extension(&mut Default::default(), "ogg"),
                mss,
                &Default::default(),
                &Default::default(),
            )?
            .format;
        let Some(track) = fmt.default_track() else {
            return Err(NoTracksError);
        };
        let mut decoder =
            symphonia::default::get_codecs().make(&track.codec_params, &Default::default())?;
        let track_id = track.id;
        let mut buf = None;
        // Skip the early parts of 2^n Vec growth to save on heap allocations
        let mut out_vec = Vec::with_capacity(256);
        loop {
            let Ok(packet) = fmt.next_packet() else {
                break
            };
            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    if packet.track_id() != track_id {
                        continue;
                    }
                    let spec = *audio_buf.spec();
                    if buf.is_none() {
                        buf = Some(SampleBuffer::<i16>::new(
                            (audio_buf.capacity()) as u64,
                            spec,
                        ))
                    }
                    if let Some(buf) = &mut buf {
                        buf.copy_interleaved_ref(audio_buf);
                        let samples = buf.samples();
                        let count = spec.channels.count();
                        for chunk in samples.chunks(count) {
                            let mut accum = 0isize;
                            for sample in chunk {
                                accum += *sample as isize;
                            }
                            accum /= count as isize;
                            //TODO: look into this overflowing
                            out_vec.push(accum as i16);
                        }
                    }
                }
                Err(e) => match e {
                    ResetRequired => decoder.reset(),
                    DecodeError(_) => break,
                    IoError(e) => match e.kind() {
                        std::io::ErrorKind::UnexpectedEof => break,
                        _ => return Err(e.into()),
                    },
                    e => return Err(e.into()),
                },
            }
        }
        // If we can't tell the sample rate, fallback to 48000hz
        let _sample_rate = decoder.codec_params().sample_rate.unwrap_or(48_000u32);

        let out_vec = {
            if let Some(sample_rate) = decoder.codec_params().sample_rate {
                convert(
                    sample_rate,
                    48_000u32,
                    1,
                    ConverterType::SincBestQuality,
                    out_vec
                        .as_slice()
                        .into_iter()
                        .map(|x| <i16 as IntoSample<f32>>::into_sample(*x))
                        .collect::<Vec<f32>>()
                        .as_slice(),
                )?
                .into_iter()
                .map(|x| <f32 as IntoSample<i16>>::into_sample(x))
                .collect::<Vec<i16>>()
            }
            //Assume we have 48khz audio if a sample rate is not provided by the decoder.
            else {
                out_vec
            }
        };
        Ok(Self(out_vec))
    }
}
struct StaticAudioProvider<T: StaticResourceProvider> {
    resource_provider: T,
    buffer: Vec<u8>,
    cache: LruCache<ResourcePath, Rc<StaticSound>>,
}
impl<T: StaticResourceProvider> StaticAudioProvider<T> {
    pub fn new(resource_provider: T, cfg: Option<SoundModNativeCfg>) -> Self {
        Self {
            resource_provider,
            buffer: Vec::with_capacity(16 * 1024),
            cache: LruCache::new(
                crate::interface::CONFIG
                    .get_or_init(|| cfg.unwrap_or(Default::default()))
                    .cache_size,
            ),
        }
    }
    pub(crate) fn load_static(
        &mut self,
        path: &ResourcePath,
    ) -> Result<Rc<StaticSound>, ResourceError> {
        let _ = self.resource_provider.oneshot(path, &mut self.buffer)?;
        let sound = Rc::new(StaticSound::new(&mut self.buffer)?);
        self.cache.push(path.clone(), sound.clone());
        Ok(sound)
    }
    pub(crate) fn get_or_load_static(
        &mut self,
        p: &ResourcePath,
    ) -> Result<Rc<StaticSound>, ResourceError> {
        if let Some(cached) = self.cache.get(p) {
            return Ok(cached.clone());
        }
        self.load_static(p)
    }
}

// Provides and manages providing sound data from ingame
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
