mod commands;

extern crate kira;
extern crate core;
extern crate symphonia;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::cpal::{CpalBackend, Error};
use kira::sound::FromFileError;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle, StaticSoundSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings};
use kira::tween::Tween;
use crate::commands::{InputStreamRead, InputStreamSeek, JavaInputStream, SoundCommand, SoundEditRequest, SoundInstance, SoundMessage};
use crate::SoundMessage::{AddSound, EditSound, SetGroupVolumes};
//struct to track the state and status of an individual sound.
pub struct SoundTracker {
    ins: SoundInstance,
    sound: StreamingSoundHandle<FromFileError>,
}

impl SoundTracker {
    pub fn new(mut ins: SoundInstance, ptrs: (InputStreamRead, InputStreamSeek),manager: &mut AudioManager) -> SoundTracker {
        let mut stream = ins.get_stream(ptrs);
        let sound_data = StreamingSoundData::from_seek_read(Box::new(stream),StreamingSoundSettings::new()).unwrap();
        println!("made sound data, from stream: {:?}, with manager with state: {:?}",stream, manager.state());
        SoundTracker {
            ins,
            sound: manager.play(sound_data).unwrap()
        }
    }
    pub fn proc_cmd(&mut self, cmd: SoundCommand) {
        match cmd {
            SoundCommand::Stop() => {
                self.sound.stop(Tween::default()).expect("E moment")
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
            AddSound(ins) => { self.add(ins) }
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

    fn add(&mut self, mut ins: SoundInstance) {
        let mut tracker = SoundTracker::new(ins,self.java_ptrs,&mut self.manager);
        self.trackers.insert(*&tracker.ins.uuid, tracker);
    }

}
// janky shit
#[no_mangle]
unsafe extern "C" fn init(java_seek_ptr: InputStreamSeek, java_read_ptr: InputStreamRead) -> usize {
    let (tx, rx) = channel::<SoundMessage>();
    thread::spawn(move || {
        let mut state = SoundEngineState::new((java_read_ptr,java_seek_ptr));
        for recv in rx {
            println!("received message! {:?}",&recv);
            state.process(recv);
        }
    });
    Box::into_raw(Box::new(tx)) as *mut Sender<SoundMessage> as usize
}

#[no_mangle]
unsafe extern "C" fn add_sound(ptr: usize, ins: SoundInstance) -> usize {
    let sender = (ptr as *mut Sender<SoundMessage>).read();
    sender.send(AddSound(ins)).expect("failed to send AddSound command to worker thread");
    let sender2 = sender.clone();
    Box::into_raw(Box::new(sender2)) as *mut Sender<SoundMessage> as usize
}
