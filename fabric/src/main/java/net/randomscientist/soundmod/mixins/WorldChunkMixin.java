package net.randomscientist.soundmod.mixins;

import net.minecraft.world.chunk.WorldChunk;
import net.randomscientist.soundmod.rust.SoundModNative;
import net.randomscientist.soundmod.util.AudioChunk;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(WorldChunk.class)
public class WorldChunkMixin {
    private AudioChunk chunk;

    public WorldChunkMixin(AudioChunk chunk) {
        this.chunk = chunk;
    }

    @Inject(method = "loadFromPacket(Lnet/minecraft/network/PacketByteBuf;Lnet/minecraft/nbt/NbtCompound;Ljava/util/function/Consumer;)V", at = @At("TAIL"))
    private void injected(CallbackInfo ci) {
        this.chunk = new AudioChunk();
        WorldChunk thi = (WorldChunk) (Object) this;
        this.chunk.readChunkInto(thi);
        SoundModNative.set_chunk(chunk.getBackingBuffer(), thi.getPos().x, thi.getPos().z);
    }
}
