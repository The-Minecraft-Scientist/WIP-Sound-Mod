package net.randomscientist.soundmod.natives;

import jdk.incubator.foreign.*;
import net.minecraft.client.sound.SoundInstance;
import net.randomscientist.soundmod.SoundMod;
import net.randomscientist.soundmod.util.ResourceDelegator;

import java.lang.annotation.Native;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.VarHandle;
import java.util.Map;
import java.util.Optional;
import java.util.HashMap;
import static jdk.incubator.foreign.ValueLayout.*;

public class Natives {
	public static GroupLayout RsSoundInstance = MemoryLayout.structLayout(
			JAVA_LONG.withName("uuid"),
			JAVA_INT.withName("size"),
			MemoryLayout.sequenceLayout(3,JAVA_DOUBLE).withName("position"),
			JAVA_FLOAT.withName("volume"),
			JAVA_FLOAT.withName("pitch")
	);
	public static GroupLayout RsJavaCallbacks = MemoryLayout.structLayout(
			ADDRESS.withName("read"),
			ADDRESS.withName("seek"),
			ADDRESS.withName("drop")
	);
	private static final CLinker linker = CLinker.systemCLinker();
	private static final MethodHandles.Lookup lookup = MethodHandles.lookup();
	private static final HashMap<String,FunctionDescriptor> nativeMethods = new HashMap<String,FunctionDescriptor>(Map.of(
			"init", FunctionDescriptor.ofVoid(RsJavaCallbacks),
			"add_streaming", FunctionDescriptor.ofVoid(RsSoundInstance),
			"add_static", FunctionDescriptor.ofVoid(RsSoundInstance,ADDRESS.withName("buf_ptr"),JAVA_LONG.withName("buf_size")),
			"tick", FunctionDescriptor.ofVoid()
	));
	private static final HashMap<String,MethodHandle> natives = new HashMap<String,MethodHandle>();
	public static final HashMap<String, NativeSymbol> callbacks = new HashMap<String, NativeSymbol>(Map.of(
			"readStream", tryGenerateNativeSymbol(ResourceDelegator.class,"readStream", FunctionDescriptor.of(JAVA_INT,JAVA_LONG,ADDRESS,JAVA_LONG)),
			"seekStream", tryGenerateNativeSymbol(ResourceDelegator.class, "seekStream", FunctionDescriptor.of(JAVA_LONG,JAVA_LONG,JAVA_LONG)),
			"dropResource", tryGenerateNativeSymbol(ResourceDelegator.class, "dropResource", FunctionDescriptor.ofVoid(JAVA_LONG))
	));
	private static NativeSymbol tryGenerateNativeSymbol(Class c,String name, FunctionDescriptor desc) {
		MethodHandle handle;
		try {
			handle = lookup.findStatic(c, name, CLinker.upcallType(desc));
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
		return linker.upcallStub(handle, desc, ResourceScope.globalScope());

	}
	public static MemorySegment createSoundStruct(long id, SoundInstance instance, int size) {
		MemorySegment struct = MemorySegment.allocateNative(RsSoundInstance.byteSize(), ResourceScope.globalScope());
		VarHandle uuid = ofName(RsSoundInstance,"uuid");
		VarHandle sizeh = ofName(RsSoundInstance,"size");
		VarHandle position = RsSoundInstance.varHandle(MemoryLayout.PathElement.groupElement("position"),MemoryLayout.PathElement.sequenceElement(0,1));
		VarHandle volume = ofName(RsSoundInstance,"volume");
		VarHandle pitch = ofName(RsSoundInstance,"pitch");

		uuid.set(struct,id);
		sizeh.set(struct,size);
		position.set(struct,0, instance.getX());
		position.set(struct,1, instance.getY());
		position.set(struct,2, instance.getZ());
		volume.set(struct, instance.getVolume());
		pitch.set(struct, instance.getPitch());
		return struct;
	}
	public static VarHandle ofName(GroupLayout struct, String name) {
		return struct.varHandle(MemoryLayout.PathElement.groupElement(name));
	}
	public static MemorySegment createCallbacksStruct() {
		MemorySegment struct = MemorySegment.allocateNative(RsJavaCallbacks.byteSize(), ResourceScope.globalScope());
		VarHandle read = ofName(RsJavaCallbacks, "read");
		VarHandle seek = ofName(RsJavaCallbacks, "seek");
		VarHandle drop = ofName(RsJavaCallbacks, "drop");

		read.set(struct, callbacks.get("readStream").address());
		seek.set(struct, callbacks.get("seekStream").address());
		drop.set(struct, callbacks.get("dropResource").address());
		return struct;
	}
	static {
		Lib.loadNatives();
		nativeMethods.forEach(
				(id,desc) -> {
					Optional<NativeSymbol> symbol = Lib.nativeLookup.lookup(id);
					if(symbol.isPresent()) {
						natives.put(id, linker.downcallHandle(symbol.get(), desc));
						SoundMod.LOGGER.info("found native function " + id);
						return;
					}
					throw new UnsatisfiedLinkError("failed to find and downcall native function \"" + id + "\"");
				}
		);
		try {
			getNativeHandle("init").invoke(createCallbacksStruct());
		} catch(Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static MethodHandle getNativeHandle(String id) {
		return natives.get(id);
	}
}
