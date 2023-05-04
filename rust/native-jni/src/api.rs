use crate::static_sound::JNIStaticSoundProvider;
use crossbeam::channel::Sender;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jdouble, jfloat, jint};
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::OnceCell;
use soundmod_native::interface::sound::resource::ResourcePath;
use soundmod_native::interface::{McToInterfaceMessage, SoundModInterfaceBuilder};

pub struct SenderCell(OnceCell<Sender<McToInterfaceMessage>>);
impl SenderCell {
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }
    ///Useful for one-off messages. For large/long running message, grab a fresh sender instead
    fn send(&self, msg: McToInterfaceMessage) {
        self.0
            .get()
            .expect("failed to acquire sender!")
            .send(msg)
            .expect("send failed!");
    }
    /// Useful when you need to send a lot of messages and dont want to worry about potential contention on the primary Sender
    fn get_new_sender(&self) -> Sender<McToInterfaceMessage> {
        self.0.get().expect("failed to acquire sender!").clone()
    }
    fn set(&self, val: Sender<McToInterfaceMessage>) {
        self.0.set(val).expect("failed to set value of sender");
    }
}

static SENDER: SenderCell = SenderCell::new();

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
    0
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn close_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn play_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn pause_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn resume_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn is_playing_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) -> jboolean {
    0u8
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn is_stopped_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint) -> jboolean {
    0u8
}

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
pub fn set_pitch_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, pitch: jfloat) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_looping_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, looping: jboolean) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_relative_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, relative: jboolean) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_ogg_stream_path_uuid(env: JNIEnv, _parent_class: JClass, uuid: jint, path: JString) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_custom_stream_uuid(
    env: JNIEnv,
    _parent_class: JClass,
    uuid: jint,
    audio_stream: JObject,
) {
}
