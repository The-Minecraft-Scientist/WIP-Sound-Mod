package net.randomscientist.soundmod.util;

import jdk.incubator.foreign.*;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.lang.invoke.MethodHandle;
import java.util.HashMap;

import static net.randomscientist.soundmod.natives.Natives.*;

public class ResourceDelegator {
	public static HashMap<Long, BufferedInputStream> streams = new HashMap<>();
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
	public static void dropResource(long id) {
		BufferedInputStream stream;
		stream = streams.get(id);
		if(stream != null) {
			try {
				stream.close();
			} catch (IOException e) {
				throw new RuntimeException(e);
			}
			streams.remove(id);
		}
	}
	public static int readToJavaArray(long uuid, byte[] buf) {
		try {
			return streams.get(uuid).read(buf);
		} catch(Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static void addResource(ResourceManager manager, Identifier id, long uuid) {
		InputStream s0;
		BufferedInputStream s;
		try {
			s0 = manager.open(id);
			int len = s0.available();
			s = new BufferedInputStream(s0,len+8191);
			s.mark(len+8190);
		} catch (Throwable e) {
			throw new RuntimeException(e);
		}
		streams.put(uuid, s);
	}
	public static int getSize(long id) {
		try {
			return streams.get(id).available();
		} catch (IOException e) {
			throw new RuntimeException(e);
		}
	}
	public static void tryLoadStatic(MemorySegment struct, MemoryAddress buf_ptr, long size) {
		try {
			MethodHandle addStaticHandle = getNativeHandle("add_static");
			addStaticHandle.invoke(struct, buf_ptr, size);
		} catch(Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static void tryLoadStreaming(MemorySegment struct) {
		MethodHandle addStreamingHandle = getNativeHandle("add_streaming");
		try {
			addStreamingHandle.invoke(struct);
		} catch(Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static void tryTick() {
		MethodHandle tickHandle = getNativeHandle("tick");
		try {
			tickHandle.invoke();
		} catch(Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static MemorySegment allocBytes(byte[] buf) {
		MemorySegment cpy_frm = MemorySegment.ofArray(buf);
		MemorySegment dst = MemorySegment.allocateNative(buf.length, ResourceScope.globalScope());
		dst.copyFrom(cpy_frm);
		return dst;
	}

}
