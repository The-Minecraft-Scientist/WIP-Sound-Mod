use crate::static_sound::JNIStaticSoundProvider;
use crossbeam::channel::{Receiver, Sender};
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jdouble, jfloat, jint, jobject};
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::OnceCell;
use soundmod_native::interface::sound::resource::ResourcePath;
use soundmod_native::interface::McToInterfaceMessage::Change;
use soundmod_native::interface::{
    InterfaceToMcTalkBack, McToInterfaceMessage, SoundModInterfaceBuilder, SoundUpdateType,
    UpdateSound,
};
use soundmod_native::playback::Player;
use std::ops::Deref;

#[derive(Debug)]
pub struct GlobalState(
    Sender<McToInterfaceMessage>,
    Receiver<InterfaceToMcTalkBack>,
);

pub struct StateCell(OnceCell<GlobalState>);
impl StateCell {
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }
    ///Useful for one-off messages. For large/long running message, grab a fresh sender instead
    fn send(&self, msg: McToInterfaceMessage) {
        self.0
            .get()
            .expect("failed to acquire sender!")
            .0
            .send(msg)
            .expect("send failed!");
    }
    /// Useful when you need to send a lot of messages and dont want to worry about potential contention on the primary Sender
    fn get_new_sender(&self) -> Sender<McToInterfaceMessage> {
        self.0.get().expect("failed to acquire sender!").0.clone()
    }
    fn set(
        &self,
        vals: (
            Sender<McToInterfaceMessage>,
            Receiver<InterfaceToMcTalkBack>,
        ),
    ) {
        self.0
            .set(GlobalState(vals.0, vals.1))
            .expect("failed to set value of global state");
    }
    fn update_sound(&self, id: u32, update: SoundUpdateType) {
        self.send(Change(UpdateSound::new(id, update)));
    }
    fn receive(&self) -> InterfaceToMcTalkBack {
        self.0
            .get()
            .expect("failed to get global state")
            .1
            .recv()
            .expect("failed to receive data from interface thread")
    }
}

static SENDER: StateCell = StateCell::new();

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn say_hi(_env: JNIEnv) {
    println!("hi!")
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn init(env: JNIEnv, _parent_class: JClass, resource_class: JClass) {
    let global = env.new_global_ref(resource_class).unwrap();
    let builder = SoundModInterfaceBuilder::new(
        JNIStaticSoundProvider::new(
            env.get_java_vm()
                .expect("failed to get JavaVM while initalizing"),
            global,
        ),
        (),
    );
    SENDER.set(builder.run())
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn get_sound_data(mut env: JNIEnv, _parent_class: JClass, id: JObject) {
    let path = ResourcePath(env.get_string(&JString::from(id)).unwrap().into());
    SENDER.send(McToInterfaceMessage::PrintSoundData(path));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn new_sound_uuid(env: JNIEnv, _parent_class: JClass) -> jint {
    SENDER.send(McToInterfaceMessage::NewSound);
    match SENDER.receive() {
        InterfaceToMcTalkBack::NewSound(i) => i as jint,
        _ => {
            panic!("received wrong talkback method")
        }
    }
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn close_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn play_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {
    SENDER.send(Change(UpdateSound::new(uuid as u32, SoundUpdateType::Play)))
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn pause_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::Pause,
    )))
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn resume_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::Resume,
    )))
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn is_playing_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) -> jboolean {
    SENDER.send(McToInterfaceMessage::Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::CheckIsPlaying,
    )));
    match SENDER.receive() {
        InterfaceToMcTalkBack::IsPlaying(b) => {
            if b {
                1u8
            } else {
                0u8
            }
        }
        _ => {
            panic!("recieved wrong talkback type")
        }
    }
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn is_stopped_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) -> jboolean {
    0u8
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn stop_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_position_uuid(
    env: JNIEnv,
    _parent_class: JClass,
    uuid: jint,
    x: jdouble,
    y: jdouble,
    z: jdouble,
) {
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_pitch_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, pitch: jfloat) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::SetPitch(pitch),
    )));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_looping_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, looping: jboolean) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::SetLooping(if looping == 0 { false } else { true }),
    )));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_relative_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, relative: jboolean) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::SetRelative(if relative == 0 { false } else { true }),
    )));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_ogg_stream_path_uuid(
    mut env: JNIEnv,
    _parent_class: JClass,
    uuid: jint,
    path: JResourcePath,
) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::SetPath(path.into(&mut env)),
    )))
}
#[repr(C)]
pub struct JResourcePath<'a>(JString<'a>);
impl<'a> JResourcePath<'a> {
    fn into(self, env: &mut JNIEnv) -> ResourcePath {
        ResourcePath(
            env.get_string(&self.0)
                .expect("failed to get resourcepath string")
                .to_str()
                .expect("failed to convert to &str")
                .to_string(),
        )
    }
}
// Still a no-op :))))))))))))))
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_custom_stream_uuid(
    env: JNIEnv,
    _parent_class: JClass,
    uuid: jint,
    audio_stream: JObject,
) {
}
