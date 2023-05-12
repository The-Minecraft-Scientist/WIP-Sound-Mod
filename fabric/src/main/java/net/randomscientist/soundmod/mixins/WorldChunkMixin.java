package net.randomscientist.soundmod.mixins;

import net.minecraft.world.chunk.WorldChunk;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(WorldChunk.class)
public class WorldChunkMixin {
    void init_audio_chunk(WorldChunk chunk) {

    }
}
