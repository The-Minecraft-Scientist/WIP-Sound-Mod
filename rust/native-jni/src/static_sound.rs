use jni::objects::{GlobalRef, JClass, JObject, JPrimitiveArray, JString, JValueGen, ReleaseMode};
use jni::strings::JNIString;

use jni::{JNIEnv, JavaVM};

use soundmod_native::interface::sound::resource::{
    ResourceError, ResourcePath, StaticResourceProvider,
};

use jni::sys::jclass;
use std::slice;

#[derive(Debug)]
struct JavaStructures {
    provider_class: GlobalRef,
    buffer: GlobalRef,
}
#[derive(Debug)]
pub struct JNIStaticSoundProvider {
    jvm: JavaVM,
    provider_class: GlobalRef,
}
impl JNIStaticSoundProvider {
    pub fn new(jvm: JavaVM, provider_class: GlobalRef) -> Self {
        Self {
            jvm,
            provider_class,
        }
    }
    fn get_env(&self) -> JNIEnv {
        return self.jvm.get_env().expect("unexpectedly detached from JVM");
    }

    fn get_stream(&mut self, path: &ResourcePath) -> GlobalRef {
        let env = &mut self.get_env();
        let string = env
            .new_string(JNIString::from(&path.0))
            .expect("Failed to create new Java String");
        let string = JValueGen::Object(<JString as AsRef<JObject<'_>>>::as_ref(&string));
        let v = env
            .call_static_method(
                &self.provider_class,
                "getResourceStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                &[string],
            )
            .expect("failed to call JVM method")
            .l()
            .expect("failed to cast result object");
        env.new_global_ref(v).unwrap()
    }
}

impl StaticResourceProvider for JNIStaticSoundProvider {
    fn oneshot<'local>(
        &mut self,
        id: &ResourcePath,
        buffer: &mut Vec<u8>,
    ) -> Result<(), ResourceError> {
        let stream = self.get_stream(id);
        let env = &mut self.get_env();
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
