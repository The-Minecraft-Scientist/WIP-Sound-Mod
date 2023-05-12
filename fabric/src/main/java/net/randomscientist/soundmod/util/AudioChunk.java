package net.randomscientist.soundmod.util;

import net.minecraft.world.chunk.WorldChunk;

import java.nio.ByteBuffer;

public class AudioChunk {

    private final ByteBuffer mref_buf;
    private WorldChunk chunk;
    AudioChunk(WorldChunk chunk) {
        this.chunk = chunk;
        this.mref_buf = ByteBuffer.allocateDirect(Constants.CHUNK_MREF_BUF_SIZE);
    }
    public void setChunk(WorldChunk chunk) {
        this.mref_buf.clear();
        this.chunk = chunk;
    }

}
