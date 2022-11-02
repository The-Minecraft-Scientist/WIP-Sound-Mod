package net.randomscientist.soundmod.natives;

import jdk.incubator.foreign.*;
import net.randomscientist.soundmod.SoundMod;
import net.randomscientist.soundmod.util.ResourceDelegator;

import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.util.Map;
import java.util.Optional;
import java.util.HashMap;
import static jdk.incubator.foreign.ValueLayout.*;

public class Natives {
	public static GroupLayout RsSoundInstance = MemoryLayout.structLayout(
			JAVA_LONG.withName("uuid"),
			MemoryLayout.sequenceLayout(3,JAVA_DOUBLE).withName("position"),
			JAVA_FLOAT.withName("volume"),
			JAVA_FLOAT.withName("pitch"),
			JAVA_BOOLEAN.withName("attenuate"),
			JAVA_BOOLEAN.withName("playing")

	);
	private static final CLinker linker = CLinker.systemCLinker();
	private static final HashMap<String,FunctionDescriptor> nativeMethods = new HashMap<String,FunctionDescriptor>(Map.of(
			"test_fn1", FunctionDescriptor.of(JAVA_FLOAT),
			"test_fn2", FunctionDescriptor.ofVoid(ADDRESS,JAVA_INT),
			"play_input_stream", FunctionDescriptor.ofVoid(JAVA_LONG,ADDRESS, ADDRESS, JAVA_INT)
	));
	private static final HashMap<String,MethodHandle> natives = new HashMap<String,MethodHandle>();
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
	}
	public static MethodHandle getNativeHandle(String id) {
		return natives.get(id);
	}
	private static final MethodHandles.Lookup lookup = MethodHandles.lookup();
	private static NativeSymbol tryGenerateNativeSymbol(Class c,String name, FunctionDescriptor desc) {
		MethodHandle handle;
		try {
			handle = lookup.findStatic(c, name, CLinker.upcallType(desc));
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
		return linker.upcallStub(handle, desc, ResourceScope.globalScope());

	}
	private static final HashMap<String,NativeSymbol> methods = new HashMap<String,NativeSymbol>(Map.of(
			"readStream", tryGenerateNativeSymbol(ResourceDelegator.class,"readStream",FunctionDescriptor.of(JAVA_INT,JAVA_LONG,ADDRESS,JAVA_LONG)),
			"seekStream", tryGenerateNativeSymbol(ResourceDelegator.class, "seekStream",FunctionDescriptor.of(JAVA_LONG,JAVA_LONG,JAVA_LONG))
	));
	public static NativeSymbol getMethodSymbol(String id) {
		return  methods.get(id);
	}
}
