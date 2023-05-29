package net.randomscientist.soundmod.mixins;

import net.minecraft.client.sound.AudioStream;
import net.minecraft.client.sound.SoundLoader;
import net.minecraft.util.Identifier;
import net.minecraft.util.Util;
import net.randomscientist.soundmod.sound.RustOggStream;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.util.concurrent.CompletableFuture;

@Mixin(SoundLoader.class)
public class SoundLoaderMixin {
    @Overwrite
    public CompletableFuture<AudioStream> loadStreamed(Identifier id, boolean repeatInstantly) {
        return CompletableFuture.supplyAsync(() -> new RustOggStream(id), Util.getMainWorkerExecutor());
    }
}
