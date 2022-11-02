use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::io::SeekFrom::{Current, End, Start};

//See natives/Natives/RsSoundInstance
#[repr(C)]
pub struct SoundInstance {
    pub uuid: u64,
    pub position: [f64; 3],
    pub volume: f32,
    pub pitch: f32,
    pub attenuate: bool,
    pub playing: bool,
}
pub enum SoundMessage {
    AddSound(SoundInstance),
    EditSound(SoundEditRequest),
    SetGroupVolumes(HashMap<i32,f32>)
}

#[repr(C)]
pub struct SoundEditRequest {
    pub uuid: u64,
    pub command: SoundCommand,
}
pub enum SoundCommand {
    ChangeVolume(f32),
    ChangePitch(f32),
    ChangeLocation([f64;3]),
    Play(),
    Pause(),
    Stop(),
}
#[derive(Copy, Clone)]
pub struct JavaInputStream {
    pub uuid: i64,
    pub read_ptr: InputStreamRead,
    pub seek_ptr: InputStreamSeek,
    pub size: i32,
    pub position: u64,
}
impl Read for JavaInputStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize,std::io::Error> {
        let res = (self.read_ptr)(self.uuid,buf.as_mut_ptr(),buf.len());
        if res>0 {
            Ok(res as usize)
        } else {
            Ok(0)
        }
    }
}

impl Seek for JavaInputStream {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64,std::io::Error>{
        match pos {
            Start(pos) => {
                self.position = (self.seek_ptr)(self.uuid,pos as i64) as u64
            }
            Current(off)=> {
                let mut npos = self.position as i64 + off;
                if npos<0 {npos = 0}
                self.position = (self.seek_ptr)(self.uuid,npos) as u64
            }
            End(off) => {
                let mut npos = self.size as i64 + off;
                if npos<0 {npos = 0}
                self.position = (self.seek_ptr)(self.uuid,npos) as u64
            }
        }
        return Ok(self.position);
    }
}
pub type InputStreamRead = extern fn(i64,*mut u8,usize) -> i32;
pub type InputStreamSeek = extern fn(i64,i64) -> i64;