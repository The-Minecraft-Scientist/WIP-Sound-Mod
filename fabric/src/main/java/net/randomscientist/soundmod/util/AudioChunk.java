package net.randomscientist.soundmod.util;

import net.minecraft.block.BlockState;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.WorldChunk;
import net.randomscientist.soundmod.mixins.ChunkSectionAccessor;
import net.randomscientist.soundmod.mixins.PalettedContainerAccessor;

import java.nio.ByteBuffer;

public class AudioChunk {

    private final ByteBuffer mref_buf;
    public AudioChunk() {
        this.mref_buf = ByteBuffer.allocateDirect(Constants.CHUNK_MREF_BUF_BYTE_SIZE);
        this.mref_buf.order(java.nio.ByteOrder.nativeOrder());
    }
    public void readChunkInto(WorldChunk chunk) {
        this.mref_buf.clear();
        ChunkSection[] sections = chunk.getSectionArray();
        long current;
        for(int i = 0; i < 24; i++) {
            if(sections[i] == null) {
                this.mref_buf.asShortBuffer().put(Constants.EMPTY_CHUNK_SECTION);
                this.mref_buf.position(this.mref_buf.position() + 4096 * 2);
                continue;
            }
            ChunkSectionAccessor thisSection = (ChunkSectionAccessor) sections[i];
            // If this chunk is empty, write air to it.
            if((thisSection.getNonEmptyBlockCount() == 0 && thisSection.getNonEmptyFluidCount() == 0)) {
                this.mref_buf.asShortBuffer().put(Constants.EMPTY_CHUNK_SECTION);
                // 1 ChunkSection == 4096 shorts. 1 short = 2 bytes.
                this.mref_buf.position(this.mref_buf.position() + 4096 * 2);
                continue;
            }
            //this cast hopefully shouldn't fail
            @SuppressWarnings("unchecked cast")
            PalettedContainerAccessor<BlockState> container = (PalettedContainerAccessor<BlockState>) thisSection.getBlockStateContainer();
            for(int j = 0; j < 4096; j += 4) {

                //explicitly vectorize an inner 4-iteration for loop needed to fill current with data
                current = makeMatIndex(container.invokeGet(j)) |
                        ( makeMatIndex(container.invokeGet(j + 1)) << 16 ) |
                        ( makeMatIndex(container.invokeGet(j + 2)) << 32 ) |
                        ( makeMatIndex(container.invokeGet(j + 3)) << 48 );
                //write the accumulated 4 shorts of data out
                this.mref_buf.asLongBuffer().put(current);
            }
            this.mref_buf.position(this.mref_buf.position() + 4096 * 2);
        }

    }
    //ensure this is FAST and COMPACT!!! IT CAN BE CALLED 98304 times PER CHUNK

    //Moving this to Rust side is an option but I'm ignoring it for now because it would be really annoying to implement
    // Additionally, the overhead from sending string block state ids in an Object[] instead of a tidy direct byte buffer I can get a raw pointer into
    // probably outweighs the potential performance gains.
    private long makeMatIndex(BlockState b) {
        if(b.isAir()) {return 0L;}
        return 1L;
    }

    //Called from native code
    @SuppressWarnings("unused")
    public ByteBuffer getBackingBuffer() {
        return this.mref_buf;
    }
}
