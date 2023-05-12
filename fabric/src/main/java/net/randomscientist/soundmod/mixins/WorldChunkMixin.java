package net.randomscientist.soundmod.mixins;

import net.minecraft.world.chunk.WorldChunk;
import net.randomscientist.soundmod.util.AudioChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(WorldChunk.class)
public class WorldChunkMixin {
    private AudioChunk audioChunk = new AudioChunk();
    public AudioChunk getAudioChunk() {
        return this.audioChunk;
    }
    @Inject(method = "loadFromPacket", at = @At("TAIL"))
    private void injected(CallbackInfo ci) {
        this.audioChunk = new AudioChunk();
        this.audioChunk.readChunkInto((WorldChunk) (Object) this);
    }
}
