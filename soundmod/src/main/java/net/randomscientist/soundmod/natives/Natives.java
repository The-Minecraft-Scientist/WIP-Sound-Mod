package net.randomscientist.soundmod.natives;

import jdk.incubator.foreign.*;
import net.randomscientist.soundmod.SoundMod;
import net.randomscientist.soundmod.util.ResourceDelegator;

import java.lang.annotation.Native;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
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
	private static final CLinker linker = CLinker.systemCLinker();
	private static final MethodHandles.Lookup lookup = MethodHandles.lookup();
	private static final HashMap<String,FunctionDescriptor> nativeMethods = new HashMap<String,FunctionDescriptor>(Map.of(
			"init", FunctionDescriptor.of(ADDRESS, ADDRESS.withName("seek_ptr"),ADDRESS.withName("read_ptr")),
			"add_sound", FunctionDescriptor.of(ADDRESS,ADDRESS.withName("sender"),RsSoundInstance)
	));
	private static final HashMap<String,MethodHandle> natives = new HashMap<String,MethodHandle>();
	public static final HashMap<String, NativeSymbol> methods = new HashMap<String, NativeSymbol>(Map.of(
			"readStream", tryGenerateNativeSymbol(ResourceDelegator.class,"readStream",FunctionDescriptor.of(JAVA_INT,JAVA_LONG,ADDRESS,JAVA_LONG)),
			"seekStream", tryGenerateNativeSymbol(ResourceDelegator.class, "seekStream",FunctionDescriptor.of(JAVA_LONG,JAVA_LONG,JAVA_LONG))
	));
	public static MemoryAddress sender;
	private static NativeSymbol tryGenerateNativeSymbol(Class c,String name, FunctionDescriptor desc) {
		MethodHandle handle;
		try {
			handle = lookup.findStatic(c, name, CLinker.upcallType(desc));
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
		return linker.upcallStub(handle, desc, ResourceScope.globalScope());

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
			sender = (MemoryAddress) natives.get("init").invoke(methods.get("seekStream").address(),methods.get("readStream").address());
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static MethodHandle getNativeHandle(String id) {
		return natives.get(id);
	}
	public static NativeSymbol getMethodSymbol(String id) {
		return  methods.get(id);
	}
}
