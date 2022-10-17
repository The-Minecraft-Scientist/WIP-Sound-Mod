package net.randomscientist.soundmod.natives;

import jdk.incubator.foreign.CLinker;
import jdk.incubator.foreign.FunctionDescriptor;
import jdk.incubator.foreign.NativeSymbol;
import net.randomscientist.soundmod.SoundMod;

import java.lang.invoke.MethodHandle;
import java.util.Map;
import java.util.Optional;
import java.util.HashMap;
import static jdk.incubator.foreign.ValueLayout.*;

public class Natives {
    private static final HashMap<String,FunctionDescriptor> nativeMethods = new HashMap<String,FunctionDescriptor>(Map.of(
            "test_fn1", FunctionDescriptor.of(JAVA_FLOAT),
            "test_fn2", FunctionDescriptor.ofVoid(ADDRESS,JAVA_INT)
    ));
    private static final HashMap<String,MethodHandle> natives = new HashMap<String,MethodHandle>();
    static {
        Lib.loadNatives();
        CLinker linker = CLinker.systemCLinker();
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

}
