use crate::interface::sound::resource::ResourceError::*;
use crate::interface::sound::resource::{ResourceError, ResourcePath, StaticResourceProvider};
use crate::interface::SoundModNativeCfg;
use lru::LruCache;
use samplerate_rs::{convert, ConverterType};
use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::ops::{Deref, DerefMut, Sub};
use std::rc::Rc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::conv::IntoSample;
use symphonia::core::errors::Error::{DecodeError, IoError, ResetRequired};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

#[derive(Clone, Debug)]
pub struct StaticSound<const BLOCK_LENGTH: usize = 256>(pub(crate) Vec<i16>);
impl<const BLOCK_LENGTH: usize> StaticSound<BLOCK_LENGTH> {
    fn new(source: &mut Vec<u8>) -> Result<Self, ResourceError> {
        // Icky clone. This is the cost of dynamic dispatch ig.
        let mss =
            MediaSourceStream::new(Box::new(Cursor::new(source.to_owned())), Default::default());
        let mut fmt = symphonia::default::get_probe()
            .format(
                Hint::with_extension(&mut Default::default(), "ogg"),
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
        //TODO: this is bad
        let _sample_rate = decoder.codec_params().sample_rate.unwrap_or(48_000u32);

        let mut out_vec = {
            if let Some(sample_rate) = decoder.codec_params().sample_rate {
                convert(
                    sample_rate,
                    48_000u32,
                    1,
                    ConverterType::SincBestQuality,
                    out_vec
                        .as_slice()
                        .iter()
                        .map(|x| <i16 as IntoSample<f32>>::into_sample(*x))
                        .collect::<Vec<f32>>()
                        .as_slice(),
                )?
                .into_iter()
                .map(<f32 as IntoSample<i16>>::into_sample)
                .collect::<Vec<i16>>()
            }
            //Assume we have 48khz audio if a sample rate is not provided by the decoder.
            else {
                out_vec
            }
        };
        let r = BLOCK_LENGTH - out_vec.len() % BLOCK_LENGTH;
        //Pad output to a multiple of block length
        out_vec.resize(out_vec.len() + r, 0);
        Ok(Self(out_vec))
    }
}
impl<const BLOCK_LENGTH: usize> Deref for StaticSound<BLOCK_LENGTH> {
    type Target = Vec<i16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<const BLOCK_LENGTH: usize> DerefMut for StaticSound<BLOCK_LENGTH> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
pub(crate) struct StaticAudioProvider<T: StaticResourceProvider, const BLOCK_LENGTH: usize = 256> {
    resource_provider: T,
    buffer: Vec<u8>,
    pub cache: LruCache<ResourcePath, Rc<StaticSound<BLOCK_LENGTH>>>,
}
impl<_T: StaticResourceProvider, const _L: usize> Debug for StaticAudioProvider<_T, _L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(
            format!(
                "StaticAudioProvider. Buffer size: {}, cache :{:?}",
                self.buffer.len(),
                self.cache
            )
            .as_str(),
        );
        Ok(())
    }
}
impl<T: StaticResourceProvider, const BLOCK_LENGTH: usize> StaticAudioProvider<T, BLOCK_LENGTH> {
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
    ) -> Result<Rc<StaticSound<BLOCK_LENGTH>>, ResourceError> {
        self.resource_provider.oneshot(path, &mut self.buffer)?;
        let sound = Rc::new(StaticSound::<BLOCK_LENGTH>::new(&mut self.buffer)?);
        self.cache.push(path.clone(), Rc::clone(&sound));
        Ok(sound)
    }
    pub(crate) fn get_or_load_static<const BLOCK_SIZE: usize>(
        &mut self,
        p: &ResourcePath,
    ) -> Result<Rc<StaticSound<BLOCK_LENGTH>>, ResourceError> {
        if let Some(cached) = self.cache.get(p) {
            return Ok(cached.clone());
        }
        self.load_static(p)
    }
}
