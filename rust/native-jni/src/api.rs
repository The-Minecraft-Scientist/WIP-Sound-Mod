use crate::static_sound::JNIStaticSoundProvider;
use jni::objects::{JClass, JObject, JString};
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::OnceCell;
use soundmod_native::interface::sound::resource::ResourcePath;
use soundmod_native::interface::{McToInterfaceMessage, SoundModInterfaceBuilder};
use std::sync::mpsc::{Sender, SyncSender};

pub struct SenderCell(OnceCell<SyncSender<McToInterfaceMessage>>);
impl SenderCell {
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }
    fn send(&self, msg: McToInterfaceMessage) {
        self.0
            .get()
            .expect("failed to acquire sender!")
            .clone()
            .send(msg)
            .expect("send failed!");
    }
    fn set(&self, val: SyncSender<McToInterfaceMessage>) {
        self.0.set(val).expect("failed to set value of sender");
    }
}

static SENDER: SenderCell = SenderCell::new();

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn say_hi(_env: JNIEnv) {
    println!("hi!")
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn init(env: JNIEnv, _class: JClass) {
    let builder = SoundModInterfaceBuilder::new(
        JNIStaticSoundProvider::new(
            env.get_java_vm()
                .expect("failed to get JavaVM while initalizing"),
        ),
        (),
    );
    SENDER.set(builder.run())
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn get_sound_data(mut env: JNIEnv, _class: JClass, id: JObject) {
    let path = ResourcePath(env.get_string(&JString::from(id)).unwrap().into());
    SENDER.send(McToInterfaceMessage::PrintSoundData(path));
}
