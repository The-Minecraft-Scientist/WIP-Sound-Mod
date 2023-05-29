use crate::static_sound::JNIStaticSoundProvider;
use crossbeam::channel::{Receiver, Sender};
use glam::{I64Vec2, IVec2};
use jni::objects::{JByteBuffer, JClass, JObject, JString};
use jni::sys::{jboolean, jdouble, jfloat, jint, jlong};
use jni::JNIEnv;
use jni_fn::jni_fn;
use once_cell::sync::OnceCell;
use soundmod_native::gpu::trace::chunk::Chunk;
use soundmod_native::gpu::trace::world::{ChunkSectionLocation, WorldChange};
use soundmod_native::gpu::{DebugRenderer, InterfaceToGpuMessage};
use soundmod_native::interface::sound::resource::ResourcePath;
use soundmod_native::interface::McToInterfaceMessage::Change;
use soundmod_native::interface::{
    InterfaceToMcTalkBack, McToInterfaceMessage, SoundModInterfaceBuilder, SoundUpdateType,
    UpdateSound,
};
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub struct InterfaceGlobalState(
    Sender<McToInterfaceMessage>,
    Receiver<InterfaceToMcTalkBack>,
);
#[derive(Debug)]
pub struct GpuGlobalState(Sender<InterfaceToGpuMessage>);
pub struct StateCell {
    interface_state: OnceCell<InterfaceGlobalState>,
    gpu_state: OnceCell<GpuGlobalState>,
}
impl StateCell {
    pub const fn new() -> Self {
        Self {
            interface_state: OnceCell::new(),
            gpu_state: OnceCell::new(),
        }
    }

    fn send(&self, msg: McToInterfaceMessage) {
        self.interface_state
            .get()
            .expect("failed to acquire sender!")
            .0
            .send(msg)
            .expect("send failed!");
    }
    fn get_new_sender(&self) -> Sender<McToInterfaceMessage> {
        self.interface_state
            .get()
            .expect("failed to acquire sender!")
            .0
            .clone()
    }
    fn set_interface(
        &self,
        vals: (
            Sender<McToInterfaceMessage>,
            Receiver<InterfaceToMcTalkBack>,
        ),
    ) {
        self.interface_state
            .set(InterfaceGlobalState(vals.0, vals.1))
            .expect("failed to set value of global state");
    }
    fn update_sound(&self, id: u32, update: SoundUpdateType) {
        self.send(Change(UpdateSound::new(id, update)));
    }
    fn receive(&self) -> InterfaceToMcTalkBack {
        let state = self
            .interface_state
            .get()
            .expect("failed to get global state");
        let res = state.1.recv().unwrap();
        res
    }
    fn set_gpu(&self, val: GpuGlobalState) {
        self.gpu_state
            .set(val)
            .expect("failed to set value of global state")
    }
    fn send_gpu(&self, msg: InterfaceToGpuMessage) {
        self.gpu_state
            .get()
            .expect("failed to acquire sender")
            .0
            .send(msg)
            .expect("failed to send message to GPU thread")
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
                .expect("failed to get JavaVM while initializing"),
            global,
        ),
        (),
    );
    SENDER.set_interface(builder.run());
    let (tx, rx) = crossbeam::channel::unbounded::<InterfaceToGpuMessage>();
    SENDER.set_gpu(GpuGlobalState(tx));
    let _ = thread::spawn(move || {
        println!("starting debug renderer");
        let mut renderer = pollster::block_on(DebugRenderer::new());
        for msg in rx {
            renderer.process(msg)
        }
    });
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn get_sound_data(mut env: JNIEnv, _parent_class: JClass, id: JObject) {
    let path = ResourcePath(env.get_string(&JString::from(id)).unwrap().into());
    SENDER.send(McToInterfaceMessage::PrintSoundData(path));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn new_sound_uuid(_env: JNIEnv, _parent_class: JClass) -> jint {
    SENDER.send(McToInterfaceMessage::NewSound);
    match SENDER.receive() {
        InterfaceToMcTalkBack::NewSound(i) => i as jint,
        _ => {
            panic!("received wrong talkback method")
        }
    }
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn close_uuid(_env: JNIEnv, _parent_class: JClass, _uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn play_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint) {
    SENDER.send(Change(UpdateSound::new(uuid as u32, SoundUpdateType::Play)))
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn pause_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::Pause,
    )))
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn resume_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::Resume,
    )))
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn is_playing_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint) -> jboolean {
    SENDER.send(Change(UpdateSound::new(
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
pub fn is_stopped_uuid(_env: JNIEnv, _parent_class: JClass, _uuid: jint) -> jboolean {
    0u8
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn stop_uuid(_env: JNIEnv, _parent_class: JClass, _uuid: jint) {}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_position_uuid(
    _env: JNIEnv,
    _parent_class: JClass,
    _uuid: jint,
    _x: jdouble,
    _y: jdouble,
    _z: jdouble,
) {
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_pitch_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint, pitch: jfloat) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::SetPitch(pitch),
    )));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_looping_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint, looping: jboolean) {
    SENDER.send(Change(UpdateSound::new(
        uuid as u32,
        SoundUpdateType::SetLooping(if looping == 0 { false } else { true }),
    )));
}

#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_relative_uuid(_env: JNIEnv, _parent_class: JClass, uuid: jint, relative: jboolean) {
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
    _env: JNIEnv,
    _parent_class: JClass,
    _uuid: jint,
    _audio_stream: JObject,
) {
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn run_debug_render(_env: JNIEnv, _parent: JClass) {
    SENDER.send_gpu(InterfaceToGpuMessage::RunDebugRender);
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn set_chunk(
    env: JNIEnv,
    _parent_class: JClass,
    chunk_buffer: JByteBuffer,
    chunk_x: jlong,
    chunk_z: jlong,
) {
    const SECTION_OFFSET: usize = std::mem::size_of::<u16>() * 16 * 16 * 16;
    //If this is not true we are in a bad spot...
    assert_eq!(
        env.get_direct_buffer_capacity(&chunk_buffer).unwrap(),
        (Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as usize)
    );
    let bufptr = env.get_direct_buffer_address(&chunk_buffer).unwrap() as *const u8;
    for section in 0..23 {
        let section_slice = unsafe {
            &*(slice_from_raw_parts(bufptr.add(section * SECTION_OFFSET), SECTION_OFFSET)
                as *const [u16; 16 * 16 * 16])
        };
        let mut out_slice = [0u16; 16 * 16 * 16];
        out_slice.copy_from_slice(section_slice);
        let change = WorldChange::Section {
            location: ChunkSectionLocation {
                chunk_coords: I64Vec2::new(chunk_x, chunk_z),
                section_index: section as u16,
            },
            new: Arc::new(out_slice),
        };
        SENDER.send_gpu(InterfaceToGpuMessage::WorldChange(change));
    }
}
