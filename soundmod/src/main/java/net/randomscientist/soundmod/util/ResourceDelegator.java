package net.randomscientist.soundmod.util;

import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;
import jdk.incubator.foreign.ValueLayout;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import net.randomscientist.soundmod.SoundMod;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.HashMap;

public class ResourceDelegator {
	public static HashMap<Long, BufferedInputStream> streams = new HashMap<>();
	private static int counter = 0;

	public static int readStream(long id, MemoryAddress pointer, long size) {
		byte[] arr;
		int numRead;
		MemorySegment buf = MemorySegment.ofAddress(pointer, size, ResourceScope.globalScope());
		try {
			arr = buf.toArray(ValueLayout.OfByte.JAVA_BYTE);
		} catch (Throwable e) {
			throw (new RuntimeException(e));
		}
		try {
			numRead = streams.get(id).read(arr);
		} catch(Throwable e) {
			throw(new RuntimeException(e));
		}
		MemorySegment out = MemorySegment.ofArray(arr);
		buf.copyFrom(out);
		return numRead;
	}
	public static long seekStream(long id, long pos) {
		try {
			BufferedInputStream s = streams.get(id);
			s.reset();
			return s.skip(pos);
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static long addResource(ResourceManager manager, Identifier id) {
		counter++;
		long uuid = id.hashCode() | ((long)counter) << 32;
		InputStream s0;
		BufferedInputStream s;
		try {
			s0 = manager.open(id);
			int len = s0.available();
			s = new BufferedInputStream(s0,len+8192);
			s.mark(len);
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
		streams.put(uuid, s);
		return uuid;
	}
	public static int getSize(long id) {
		try {
			return streams.get(id).available();
		} catch (IOException e) {
			throw new RuntimeException(e);
		}
	}

}
