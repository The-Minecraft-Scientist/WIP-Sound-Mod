mod commands;

extern crate kira;
extern crate core;
extern crate symphonia;


use std::collections::HashMap;
use std::io::{Cursor};


use std::sync::mpsc::{channel, Sender};
use std::{thread};


use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::{CpalBackend};
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundSettings};
use kira::tween::Tween;

use crate::commands::{InputStreamRead, InputStreamSeek, SoundCommand, SoundEditRequest, SoundHandle, SoundInstance, SoundMessage};
use crate::commands::SoundHandle::{StaticHandle, StreamingHandle};
use crate::commands::SoundMessage::{AddStatic, AddStreaming, EditSound, SetGroupVolumes};
//struct to track the state and status of an individual sound.
pub struct SoundTracker {
    ins: SoundInstance,
    sound: SoundHandle,
}

impl SoundTracker {
    pub fn add_streaming(ins: SoundInstance, ptrs: (InputStreamRead, InputStreamSeek),manager: &mut AudioManager) -> SoundTracker {
        let stream = ins.get_stream(ptrs);
        let sound_data = StreamingSoundData::from_seek_read(Box::new(stream),StreamingSoundSettings::new()).unwrap();
        println!("made sound data, from stream: {:?}, with manager with state: {:?}",stream, manager.state());
        SoundTracker {
            ins,
            sound: StreamingHandle(manager.play(sound_data).unwrap())
        }
    }
    pub fn add_static(ins: SoundInstance, manager: &mut AudioManager, buf: Vec<u8>) -> SoundTracker {
        let sound_data = StaticSoundData::from_cursor(Cursor::new(buf),StaticSoundSettings::new()).unwrap();
        SoundTracker {
            ins,
            sound: StaticHandle(manager.play(sound_data).unwrap())
        }
    }
    pub fn proc_cmd(&mut self, cmd: SoundCommand) {
        match cmd {
            SoundCommand::Stop() => {
                match &mut self.sound {
                    StaticHandle(handle) => {
                        handle.stop(Tween::default())
                    }
                    StreamingHandle(handle) => {
                        handle.stop(Tween::default())
                    }
                }.expect("failed to stop sound");
            }
            SoundCommand::ChangeLocation(l) => {
                self.ins.position = l;
            }
            SoundCommand::ChangeVolume(v) => {
                self.ins.volume = v;
            }
            _ => {}
        }
        self.update();
    }
    pub fn update(&mut self) {

    }
}

//state for the SoundEngine
struct SoundEngineState {
    trackers: HashMap<u64, SoundTracker>,
    volumes: HashMap<i32, f32>,
    java_ptrs: (InputStreamRead, InputStreamSeek),
    manager: AudioManager,
}

impl SoundEngineState {
    fn new(java_ptrs: (InputStreamRead, InputStreamSeek)) -> SoundEngineState {
        SoundEngineState {
            trackers: Default::default(),
            volumes: Default::default(),
            java_ptrs,
            manager: AudioManager::<CpalBackend>::new(AudioManagerSettings::default()).expect("oops")
        }
    }

    fn process(&mut self, msg: SoundMessage) {
        match msg {
            AddStatic(ins, buf) => { self.add_static(ins, buf) }
            AddStreaming(ins) => { self.add_streaming(ins) }
            EditSound(req) => { self.edit(req) }
            SetGroupVolumes(m) => {self.volumes = m}
        }
    }

    fn edit(&mut self, req: SoundEditRequest) {
        let tracker= match self.trackers.get_mut(&req.uuid) {
            Some(v) => {v}
            None => {
                println!("failed to get SoundTracker for uuid {}", &req.uuid);
                return;
            }
        };
        tracker.proc_cmd(req.command);
    }

    fn add_streaming(&mut self, ins: SoundInstance) {
        let tracker = SoundTracker::add_streaming(ins,self.java_ptrs,&mut self.manager);
        self.trackers.insert(*&tracker.ins.uuid, tracker);
    }
    fn add_static(&mut self, ins: SoundInstance, buf: Vec<u8>) {
        let _tracker = SoundTracker::add_static(ins, &mut self.manager, buf);
    }

}
// janky shit
#[no_mangle]
unsafe extern "C" fn init(java_seek_ptr: InputStreamSeek, java_read_ptr: InputStreamRead) -> usize {
    let (tx, rx) = channel::<SoundMessage>();
    thread::spawn(move || {
        let mut state = SoundEngineState::new((java_read_ptr,java_seek_ptr));
        for recv in rx {
            //println!("received message! {:?}",&recv);
            state.process(recv);
        }
    });
    Box::into_raw(Box::new(tx)) as *mut Sender<SoundMessage> as usize

}

#[no_mangle]
unsafe extern "C" fn add_streaming(ptr: usize, ins: SoundInstance) -> usize {
    let sender = (ptr as *mut Sender<SoundMessage>).read();
    sender.send(AddStreaming(ins)).expect("ERROR: sound thread crashed");
    let sender2 = sender.clone();
    Box::into_raw(Box::new(sender2)) as *mut Sender<SoundMessage> as usize
}

#[no_mangle]
unsafe extern "C" fn add_static(ptr: usize, ins: SoundInstance, bufptr: usize, bufsize: usize) -> usize{
    let sender = (ptr as *mut Sender<SoundMessage>).read();
    let vec = Vec::from_raw_parts(bufptr as *mut u8,bufsize,bufsize);
    sender.send(AddStatic(ins, vec)).expect("ERROR: sound thread crashed");
    let sender2 = sender.clone();
    Box::into_raw(Box::new(sender2)) as *mut Sender<SoundMessage> as usize
}
pub trait Handle {

}
