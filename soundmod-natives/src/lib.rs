extern crate libc;
extern crate rodio;

use std::io::{BufReader, Read, Seek, SeekFrom};
use std::io::SeekFrom::{Current, End, Start};
use std::thread;
use libc::size_t;
use rodio::{Decoder, OutputStream, Sink, Source};

#[no_mangle]
unsafe extern "C" fn test_fn1() -> f32 {
    println!("hello from rust!");
    return 3.0 as f32
}

#[no_mangle]
unsafe extern "C" fn test_fn2(pointer: size_t, size: i32) {
    let array = unsafe {std::slice::from_raw_parts(pointer as *const i32, size as usize)};
}
#[no_mangle]
unsafe extern "C" fn play_input_stream(uuid: i64, read_ptr: InputStreamRead, seek_ptr: InputStreamSeek, available: i32) {
    let mut input_stream = JavaInputStream {
        uuid,
        read_ptr,
        seek_ptr,
        size: available,
        position: 0 as u64,
    };
    let sound = BufReader::new(input_stream);
    let source = Decoder::new(sound).unwrap().convert_samples::<i16>();
    println!("playing sound!");
    thread::spawn(|| {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.append(source);
        sink.sleep_until_end();
    });
}
#[derive(Copy, Clone)]
pub struct JavaInputStream {
    uuid: i64,
    read_ptr: InputStreamRead,
    seek_ptr: InputStreamSeek,
    size: i32,
    position: u64,
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

type InputStreamRead = extern fn(i64,*mut u8,usize) -> i32;
type InputStreamSeek = extern fn(i64,i64) -> i64;
