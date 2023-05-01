use jni::objects::{GlobalRef, JObject, JPrimitiveArray, JString, JValueGen, ReleaseMode};
use jni::strings::JNIString;

use jni::{JNIEnv, JavaVM};

use soundmod_native::interface::sound::resource::{
    ResourceError, ResourcePath, StaticResourceProvider,
};

use std::slice;
#[derive(Debug)]
struct JavaStructures {
    provider_class: GlobalRef,
    buffer: GlobalRef,
}
#[derive(Debug)]
pub struct JNIStaticSoundProvider {
    jvm: JavaVM,
    jvm_res: Option<JavaStructures>,
}
impl JNIStaticSoundProvider {
    pub fn new(jvm: JavaVM) -> Self {
        Self { jvm, jvm_res: None }
    }
    fn get_env(&self) -> JNIEnv {
        return self.jvm.get_env().expect("unexpectedly detached from JVM");
    }
    fn get_resources(&self) -> &JavaStructures {
        let Some(res) = &self.jvm_res else {
            panic!("JVM side resources not intialized")
        };
        res
    }
    fn get_stream(&mut self, path: &ResourcePath) -> GlobalRef {
        let env = &mut self.get_env();
        let string = env
            .new_string(JNIString::from(&path.0))
            .expect("Failed to create new Java String");
        let string = JValueGen::Object(<JString as AsRef<JObject<'_>>>::as_ref(&string));
        let v = env
            .call_static_method(
                &self.get_resources().provider_class,
                "getResourceStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                &[string],
            )
            .expect("failed to call JVM method")
            .l()
            .expect("failed to cast result object");
        env.new_global_ref(v).unwrap()
    }
    fn find_class(&self) -> GlobalRef {
        let class = self
            .get_env()
            .find_class("net/randomscientist/soundmod/util/ResourceProvider")
            .expect("failed to find provider class");
        self.get_env()
            .new_global_ref(class)
            .expect("failed to create global reference to provider class")
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
        let jbuf = self.get_resources().buffer.as_raw();
        buffer.clear();
        loop {
            let int = env
                .call_method(
                    &stream,
                    "read",
                    "([B)I",
                    &[unsafe { &JObject::from_raw(jbuf) }.into()],
                )
                .expect("failed to call JVM method read")
                .i()
                .expect("failed to cast result to JInt");
            let arr = unsafe { JPrimitiveArray::<'local, u8>::from_raw(jbuf) };
            if int > 0 {
                let elements = unsafe {
                    env.get_array_elements(&arr, ReleaseMode::NoCopyBack)
                        .unwrap()
                };
                let s: &[u8] = unsafe { slice::from_raw_parts(elements.as_ptr(), int as usize) };
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
        let env = self.get_env();
        self.jvm_res = Some(JavaStructures {
            provider_class: self.find_class(),
            buffer: env
                .new_global_ref(
                    env.new_byte_array(1024)
                        .expect("failed to create java byte storage array"),
                )
                .unwrap(),
        });
    }
}
