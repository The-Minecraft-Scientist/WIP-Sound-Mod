package net.randomscientist.soundmod.util;

import net.minecraft.block.BlockState;
import net.minecraft.world.chunk.ChunkSection;
import net.minecraft.world.chunk.WorldChunk;
import net.randomscientist.soundmod.SoundMod;
import net.randomscientist.soundmod.mixins.ChunkSectionAccessor;
import net.randomscientist.soundmod.mixins.PalettedContainerAccessor;

import java.nio.ByteBuffer;

public class AudioChunk {
    public boolean read = false;
    private final ByteBuffer mref_buf;
    public AudioChunk() {
        this.mref_buf = ByteBuffer.allocateDirect(Constants.CHUNK_MREF_BUF_BYTE_SIZE);
    }
    public void readChunkInto(WorldChunk chunk) {
        this.mref_buf.clear();
        //This needs to be incremented whenever we write new things to the buffer. Needs to be 8-byte aligned
        ChunkSection[] sections = chunk.getSectionArray();
        long current;
        for(int i = 0; i < 24; i++) {
            if(sections[i] == null) {
                //TODO: track offset
                this.mref_buf.asShortBuffer().put(Constants.EMPTY_CHUNK_SECTION.clone());
                this.mref_buf.position(this.mref_buf.position() + 4096 * 2);
                continue;
            }
            ChunkSectionAccessor thisSection = (ChunkSectionAccessor) sections[i];
            // If this chunk is empty, write air to it.
            if((thisSection.getNonEmptyBlockCount() == 0 && thisSection.getNonEmptyFluidCount() == 0)) {
                //TODO: track offset
                this.mref_buf.asShortBuffer().put(Constants.EMPTY_CHUNK_SECTION.clone());
                this.mref_buf.position(this.mref_buf.position() + 4096 * 2);
                continue;
            }
            //this cast hopefully shouldn't fail
            PalettedContainerAccessor<BlockState> container = (PalettedContainerAccessor<BlockState>) thisSection.getBlockStateContainer();
            for(int j = 0; j < 4096; j += 4) {

                //explicitly vectorize an inner 4-long for loop needed to fill current with data
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
    public long makeMatIndex(BlockState b) {
        return 0L;
    }

}
