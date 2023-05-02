package net.randomscientist.soundmod.sound;

import net.minecraft.client.sound.AudioStream;

import javax.sound.sampled.AudioFormat;
import java.io.IOException;
import java.nio.ByteBuffer;

public class RustOggStream implements AudioStream {
    @Override
    public AudioFormat getFormat() {
        return null;
    }

    @Override
    public ByteBuffer getBuffer(int size) {
        return null;
    }
    @Override
    public void close() throws IOException {}
}
