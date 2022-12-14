use std::collections::HashMap;
use std::io::SeekFrom::{Current, End, Start};
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc::Sender;

use kira::sound::static_sound::StaticSoundHandle;
use kira::sound::streaming::StreamingSoundHandle;
use kira::sound::FromFileError;
use symphonia::core::io::MediaSource;
use crate::CALLBACKS;

//See natives/Natives/RsSoundInstance
#[repr(C)]
#[derive(Debug)]
pub struct SoundInstance {
    pub uuid: u64,
    pub size: i32,
    pub position: [f64; 3],
    pub volume: f32,
    pub pitch: f32,
}
impl SoundInstance {
    pub fn get_stream(&self) -> JavaInputStream {
        JavaInputStream {
            uuid: self.uuid,
            size: self.size,
            position: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct JavaCallbacks {
    pub read: InputStreamRead,
    pub seek: InputStreamSeek,
    pub drop: ResourceDelegatorDrop,
}

#[derive(Debug)]
pub enum SoundMessage {
    AddStreaming(SoundInstance),
    AddStatic(SoundInstance, Vec<u8>),
    EditSound(SoundEditRequest),
    SetGroupVolumes(HashMap<i32, f32>),
    Tick(),
}
#[derive(Debug)]
pub struct SenderWrapper {
    pub sender: Sender<SoundMessage>
}
//this is safe because SenderWrapper is always called from the thread that initialized it
unsafe impl Sync for SenderWrapper {}

pub enum SoundHandle {
    StaticHandle(StaticSoundHandle),
    StreamingHandle(StreamingSoundHandle<FromFileError>),
}

#[repr(C)]
#[derive(Debug)]
pub struct SoundEditRequest {
    pub uuid: u64,
    pub command: SoundCommand,
}
#[derive(Debug)]
pub enum SoundCommand {
    ChangeVolume(f32),
    ChangePitch(f32),
    ChangeLocation([f64; 3]),
    Play(),
    Pause(),
    Stop(),
}
#[derive(Copy, Clone, Debug)]
pub struct JavaInputStream {
    pub uuid: u64,
    pub size: i32,
    pub position: u64,
}
impl JavaInputStream {
    pub fn to_vec(&self) -> Vec<u8> {
        let read_ptr = CALLBACKS.get().unwrap().read;
        let mut buf: Vec<u8> = Vec::with_capacity(self.size as usize);
        if !(read_ptr)(self.uuid, buf.as_mut_ptr(), self.size as usize) == self.size {
            panic!("that shouldn't have happened")
        };
        buf
    }
}
impl Read for JavaInputStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let read_ptr = CALLBACKS.get().unwrap().read;
        let res = (read_ptr)(self.uuid, buf.as_mut_ptr(), buf.len());
        if res > 0 {
            Ok(res as usize)
        } else {
            Ok(0)
        }
    }
}

impl Seek for JavaInputStream {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        let seek_ptr = CALLBACKS.get().unwrap().seek;
        match pos {
            Start(pos) => self.position = (seek_ptr)(self.uuid, pos),
            Current(off) => {
                let mut npos = self.position as i64 + off;
                if npos < 0 {
                    npos = 0
                }
                self.position = (seek_ptr)(self.uuid, npos as u64)
            }
            End(off) => {
                let mut npos = self.size as i64 + off;
                if npos < 0 {
                    npos = 0
                }
                self.position = (seek_ptr)(self.uuid, npos as u64)
            }
        }
        return Ok(self.position);
    }
}
impl MediaSource for JavaInputStream {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.size as u64)
    }
}

pub type InputStreamRead = extern "C" fn(u64, *mut u8, usize) -> i32;

pub type InputStreamSeek = extern "C" fn(u64, u64) -> u64;

pub type ResourceDelegatorDrop = extern "C" fn(u64);