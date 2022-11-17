mod commands;

extern crate core;
extern crate kira;
extern crate symphonia;

use std::collections::HashMap;
use std::io::Cursor;

use std::sync::mpsc::{channel, Sender};
use std::{slice, thread};

use kira::manager::backend::cpal::CpalBackend;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::static_sound::{PlaybackState, StaticSoundData, StaticSoundSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundSettings};
use kira::tween::Tween;

use crate::commands::SoundHandle::{StaticHandle, StreamingHandle};
use crate::commands::SoundMessage::{AddStatic, AddStreaming, EditSound, SetGroupVolumes, Tick};
use crate::commands::{InputStreamRead, InputStreamSeek, JavaCallbacks, SoundCommand, SoundEditRequest, SoundHandle, SoundInstance, SoundMessage};
//struct to track the state and status of an individual sound.
pub struct SoundTracker {
    ins: SoundInstance,
    sound: SoundHandle,
}

impl SoundTracker {
    pub fn add_streaming(
        ins: SoundInstance,
        ptrs: (InputStreamRead, InputStreamSeek),
        manager: &mut AudioManager,
    ) -> SoundTracker {
        let stream = ins.get_stream(ptrs);
        let sound_data =
            StreamingSoundData::from_media_source(Box::new(stream), StreamingSoundSettings::new())
                .unwrap();
        SoundTracker {
            ins,
            sound: StreamingHandle(manager.play(sound_data).unwrap()),
        }
    }
    pub fn add_static(
        ins: SoundInstance,
        manager: &mut AudioManager,
        buf: Vec<u8>,
    ) -> SoundTracker {
        let sound_data =
            StaticSoundData::from_cursor(Cursor::new(buf), StaticSoundSettings::new()).unwrap();
        SoundTracker {
            ins,
            sound: StaticHandle(manager.play(sound_data).unwrap()),
        }
    }
    pub fn proc_cmd(&mut self, cmd: SoundCommand) {
        match cmd {
            SoundCommand::Stop() => {
                match &mut self.sound {
                    StaticHandle(handle) => handle.stop(Tween::default()),
                    StreamingHandle(handle) => handle.stop(Tween::default()),
                }
                .expect("failed to stop sound");
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
    pub fn update(&mut self) {}
}

//state for the SoundEngine
struct SoundEngineState {
    trackers: HashMap<u64, SoundTracker>,
    volumes: HashMap<i32, f32>,
    callbacks: JavaCallbacks,
    manager: AudioManager,
}

impl SoundEngineState {
    fn new(callbacks: JavaCallbacks) -> SoundEngineState {
        SoundEngineState {
            trackers: Default::default(),
            volumes: Default::default(),
            callbacks,
            manager: AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
                .expect("failed to create new AudioManager"),
        }
    }

    fn process(&mut self, msg: SoundMessage) {
        match msg {
            AddStatic(ins, buf) => self.add_static(ins, buf),
            AddStreaming(ins) => self.add_streaming(ins),
            EditSound(req) => self.edit(req),
            SetGroupVolumes(m) => self.volumes = m,
            Tick() => self.tick(),
        }
    }
    fn tick(&mut self) {
        let mut to_remove: Vec<u64> = Default::default();
        for tracker in &self.trackers {
            let state = match &tracker.1.sound {
                StaticHandle(handle) => handle.state(),
                StreamingHandle(handle) => handle.state(),
            };
            if state == PlaybackState::Stopped {
                to_remove.push(*tracker.0);
                (self.callbacks.drop)(*tracker.0);
            }
        }
    }
    fn edit(&mut self, req: SoundEditRequest) {
        let tracker = match self.trackers.get_mut(&req.uuid) {
            Some(v) => v,
            None => {
                println!("failed to get SoundTracker for uuid {}", &req.uuid);
                return;
            }
        };
        tracker.proc_cmd(req.command);
    }

    fn add_streaming(&mut self, ins: SoundInstance) {
        let tracker = SoundTracker::add_streaming(ins, (self.callbacks.read,self.callbacks.seek), &mut self.manager);
        self.trackers.insert(*&tracker.ins.uuid, tracker);
    }
    fn add_static(&mut self, ins: SoundInstance, buf: Vec<u8>) {
        let _tracker = SoundTracker::add_static(ins, &mut self.manager, buf);
    }
}
// mass abuse of
#[no_mangle]
unsafe extern "C" fn init(cbs:JavaCallbacks) -> usize {
    let (tx, rx) = channel::<SoundMessage>();
    thread::spawn(move || {
        let mut state = SoundEngineState::new(cbs);
        for recv in rx {
            state.process(recv);
        }
    });
    Box::into_raw(Box::new(tx)) as *mut Sender<SoundMessage> as usize
}

#[no_mangle]
unsafe extern "C" fn add_streaming(ptr: usize, ins: SoundInstance) -> usize {
    let sender = (ptr as *mut Sender<SoundMessage>).read();
    sender
        .send(AddStreaming(ins))
        .expect("ERROR: sound thread crashed");
    let sender2 = sender.clone();
    Box::into_raw(Box::new(sender2)) as *mut Sender<SoundMessage> as usize
}

#[no_mangle]
unsafe extern "C" fn add_static(
    ptr: usize,
    ins: SoundInstance,
    buf_ptr: usize,
    buf_size: usize,
) -> usize {
    let sender = (ptr as *mut Sender<SoundMessage>).read();
    let buf = slice::from_raw_parts(buf_ptr as *mut u8, buf_size);
    sender
        .send(AddStatic(ins, buf.to_vec()))
        .expect("ERROR: sound thread crashed");
    let sender2 = sender.clone();
    Box::into_raw(Box::new(sender2)) as *mut Sender<SoundMessage> as usize
}

#[no_mangle]
unsafe extern "C" fn tick(ptr: usize) -> usize {
    let sender = (ptr as *mut Sender<SoundMessage>).read();
    sender.send(Tick()).expect("ERROR: sound thread crashed");
    let sender2 = sender.clone();
    Box::into_raw(Box::new(sender2)) as *mut Sender<SoundMessage> as usize
}