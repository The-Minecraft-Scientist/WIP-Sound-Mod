package net.randomscientist.soundmod.sound;

import net.minecraft.client.sound.AudioStream;
import net.minecraft.util.Identifier;

import javax.sound.sampled.AudioFormat;
import java.io.IOException;
import java.nio.ByteBuffer;

public class RustOggStream implements ResourcePathAudioStream {
    private final Identifier path;
    public RustOggStream(Identifier p) {
        this.path = p;
    }
    @Override
    public Identifier get_path() {
        return this.path;
    }
    //This probably shouldn't be null, but whatever :)))
    @Override
    public AudioFormat getFormat() {
        return null;
    }
    // This is a no-op
    @Override
    public ByteBuffer getBuffer(int size) {
        return null;
    }
    //Also a no-op
    @Override
    public void close() throws IOException {}
}
