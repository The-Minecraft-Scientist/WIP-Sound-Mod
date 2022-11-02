mod commands;

extern crate libc;
extern crate rodio;

use std::collections::HashMap;
use std::io::{BufReader};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use libc::size_t;
use rodio::{Decoder, OutputStream, Sink, Source};
use crate::commands::{InputStreamRead, JavaInputStream, SoundCommand, SoundEditRequest, SoundInstance, SoundMessage};
use crate::SoundMessage::{AddSound, EditSound, SetGroupVolumes};

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
unsafe extern "C" fn play_input_stream(uuid: i64, read_ptr: InputStreamRead, seek_ptr: commands::InputStreamSeek, available: i32) {
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
struct SoundEngineState {
    senders: HashMap<u64, Sender<SoundCommand>>,
    volumes: HashMap<i32, f32>,
}
impl SoundEngineState {
    fn new() -> SoundEngineState {
        SoundEngineState {
            senders: Default::default(),
            volumes: Default::default(),
        }
    }
    fn process(&mut self, msg: SoundMessage) {
        match msg {
            AddSound(ins) => { self.add(ins) }
            EditSound(req) => { self.edit(req) }
            SetGroupVolumes(m) => {self.volumes = m}
        }
    }
    fn edit(&mut self, req: SoundEditRequest) {
        let sender = match self.senders.get(&req.uuid) {
            Some(v) => {v}
            None => {
                println!("failed to get sender for uuid {}", &req.uuid);
                return;
            }
        };
        sender.send(req.command).expect("hopefully this doesnt fail");
    }
    fn add(&mut self, ins: SoundInstance) {
        let uuid = ins.uuid.clone();
        let (tx, rx) = channel::<SoundCommand>();
        self.senders.insert(uuid,tx);
        thread::spawn(move || {

            for cmd in rx {

            }
        });
    }
}

unsafe extern "C" fn init() -> *mut Sender<SoundMessage> {
    let (mut tx, rx) = channel::<SoundMessage>();
    thread::spawn(move || {
        let mut state = SoundEngineState::new();
        for recv in rx {
            state.process(msg);
        }
    });
    &mut tx as *mut Sender<SoundMessage>
}
unsafe extern "C" fn add_sound(ptr: *mut Sender<SoundMessage>, ins: SoundInstance) -> *mut Sender<SoundMessage> {
    let mut sender = ptr.read();
    drop(ptr);
    sender.send(AddSound(ins)).expect("failed to send AddSound command to worker thread");
    &mut sender as *mut Sender<SoundMessage>
}