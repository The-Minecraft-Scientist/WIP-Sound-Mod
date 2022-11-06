package net.randomscientist.soundmod.util;

import jdk.incubator.foreign.*;
import net.minecraft.client.sound.SoundInstance;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import net.randomscientist.soundmod.natives.Natives;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.VarHandle;
import java.util.HashMap;

import static net.randomscientist.soundmod.natives.Natives.*;

public class ResourceDelegator {
	private static int counter = 0;
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
			s.mark(len);
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
	public static void tryLoadStatic(MemorySegment struct, MemoryAddress bufptr, long size) {
		try {
			MethodHandle addStaticHandle = getNativeHandle("add_static");
			sender = (MemoryAddress) addStaticHandle.invoke(sender, struct, bufptr, size);
		} catch(Throwable e) {
			throw new RuntimeException(e);
		}
	}
	public static void tryLoadStreaming(MemorySegment struct) {
		MethodHandle addStreamingHandle = getNativeHandle("add_streaming");
		try {
			sender = (MemoryAddress) addStreamingHandle.invoke(sender, struct);
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
