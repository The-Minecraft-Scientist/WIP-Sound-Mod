use jni::objects::{JByteArray, JClass, JObject, JString, JValueGen, ReleaseMode};
use jni::strings::{JNIString, JavaStr};
use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use once_cell::unsync::OnceCell;
use soundmod_native::interface::sound::resource::{
    AudioProvider, ResourceError, ResourcePath, StaticResourceProvider,
};
use std::slice;

pub struct JNIStaticSoundProvider {
    jvm: JavaVM,
}
impl JNIStaticSoundProvider {
    pub fn init(mut env: JNIEnv) -> Self {
        let jvm = env.get_java_vm().expect("failed to obtain JavaVM");
        let _ = jvm
            .attach_current_thread_as_daemon()
            .expect("failed to attach to main thread");
        Self { jvm }
    }
    fn get_env(&self) -> JNIEnv {
        return self.jvm.get_env().expect("unexpectedly detached from JVM");
    }
    fn get_stream<'f>(env: &mut JNIEnv<'f>, path: &ResourcePath) -> JObject<'f> {
        let class = env
            .find_class("net/randomscientist/soundmod/util/ResourceProvider")
            .expect("failed to find java resource provider");
        let string = env
            .new_string(JNIString::from(&path.0))
            .expect("Failed to create new Java String");
        let v = env
            .call_static_method(
                class,
                "getResourceStream",
                "(Ljava/lang/String)Ljava/io/InputStream",
                &[JValueGen::Object(<JString as AsRef<JObject<'f>>>::as_ref(
                    &string,
                ))],
            )
            .expect("failed to call native method")
            .l()
            .expect("failed to cast result object");
        v
    }
}
thread_local! {
    pub static STATE: OnceCell<AudioProvider<JNIStaticSoundProvider<>, ()>> = OnceCell::new();
}
#[jni_fn("net.randomscientist.soundmod.rust.SoundModNative")]
pub fn init(env: JNIEnv<'static>) {
    let _ = STATE.with(|x| x.set(AudioProvider::new(JNIStaticSoundProvider::init(env), ())));
}
impl StaticResourceProvider for JNIStaticSoundProvider {
    fn oneshot(&mut self, id: &ResourcePath, buffer: &mut Vec<u8>) -> Result<(), ResourceError> {
        let mut env = self.get_env();
        let stream = Self::get_stream(&mut env, id);

        // DO NOT CALL THIS ON A NON-FILE/ARRAY BACKED STREAM!!! IT WILL BLOCK UNTIL THE STREAM IS CLOSED!!
        let buf = env
            .call_method(&stream, "readAllBytes", "()[B", &[])
            .map(|x| JByteArray::from(x.l().expect("failed to cast to output")))
            .expect("failed to call readAllBytes");
        buffer.clear();
        let s = buffer.as_mut_slice();
        //SAFETY: i8 and u8 are guaranteed to have an identical layout. The `transmute`d reference does not leak outside the scope of the function call.
        let _ = env
            .get_byte_array_region(buf, 0, unsafe { std::mem::transmute(s) })
            .expect("failed to copy buffer contents");
        //TODO: graceful errors insteaad of expect spam
        Ok(())
    }
}
