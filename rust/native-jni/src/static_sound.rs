use jni::objects::{JClass, JObject, JString, JValueGen, ReleaseMode};
use jni::strings::JNIString;
use std::cell::{Ref, RefCell};

use jni::{JNIEnv, JavaVM};
use jni_fn::jni_fn;
use once_cell::sync::OnceCell;
use soundmod_native::interface::sound::resource::{
    ResourceError, ResourcePath, StaticResourceProvider,
};

use std::slice;

#[derive(Debug)]
pub struct JNIStaticSoundProvider {
    jvm: JavaVM,
}
impl JNIStaticSoundProvider {
    pub fn new(jvm: JavaVM) -> Self {
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
        let string = JValueGen::Object(<JString as AsRef<JObject<'f>>>::as_ref(&string));
        let v = env
            .call_static_method(
                class,
                "getResourceStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                &[string],
            )
            .expect("failed to call JVM method")
            .l()
            .expect("failed to cast result object");
        v
    }
}

impl StaticResourceProvider for JNIStaticSoundProvider {
    fn oneshot(&mut self, id: &ResourcePath, buffer: &mut Vec<u8>) -> Result<(), ResourceError> {
        let mut env = self.get_env();
        let stream = Self::get_stream(&mut env, id);
        let jbuf = env
            .new_byte_array(1024)
            .expect("failed to create java byte storage array");
        buffer.clear();
        loop {
            let int = env
                .call_method(
                    &stream,
                    "read",
                    "([B)I",
                    &[unsafe { &JObject::from_raw(jbuf.as_raw()) }.into()],
                )
                .expect("failed to call JVM method read")
                .i()
                .expect("failed to cast result to JInt");

            if int > 0 {
                let elements = unsafe {
                    env.get_array_elements(&jbuf, ReleaseMode::NoCopyBack)
                        .unwrap()
                };
                //SAFETY: the cast from jint to usize won't overflow, since we can only be in this branch if int > 0. Casting a *mut i8 to *const u8 does not change layout
                let s: &[u8] =
                    unsafe { slice::from_raw_parts(elements.as_ptr() as *const u8, int as usize) };
                buffer.extend_from_slice(s);
            } else {
                break;
            }
        }
        //TODO: graceful errors instead of expect spam
        Ok(())
    }
    fn init_on_thread(&mut self) {
        let Ok(_thing) = self.jvm.attach_current_thread_permanently() else {
            panic!("failed to attach interface thread to JVM!")
        };
    }
}
