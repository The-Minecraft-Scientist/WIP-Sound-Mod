use crate::static_sound::JNIStaticSoundProvider;
use crossbeam::channel::Sender;
use jni::objects::{JClass, JObject, JString};
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
    fn send(&self, msg: McToInterfaceMessage) {
        println!("sending message");
        self.0
            .get()
            .expect("failed to acquire sender!")
            .send(msg)
            .expect("send failed!");
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
    println!("running builder");
    SENDER.set(builder.run())
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn get_sound_data(mut env: JNIEnv, _class: JClass, id: JObject) {
    let path = ResourcePath(env.get_string(&JString::from(id)).unwrap().into());
    SENDER.send(McToInterfaceMessage::PrintSoundData(path));
}
