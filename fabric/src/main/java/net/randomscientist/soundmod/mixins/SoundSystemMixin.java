package net.randomscientist.soundmod.mixins;

import io.netty.util.HashedWheelTimer;
import net.minecraft.client.sound.*;
import net.minecraft.sound.SoundCategory;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.Vec3d;
import net.randomscientist.soundmod.SoundMod;
import org.spongepowered.asm.mixin.*;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

import java.util.concurrent.CompletableFuture;

@Mixin(SoundSystem.class)
public class SoundSystemMixin {
    @Shadow
    @Final
    private SoundLoader soundLoader;

    @Inject(at = @At(value = "INVOKE", target = "Lnet/minecraft/client/sound/Channel$SourceManager;run(Ljava/util/function/Consumer;)V"), method = "play(Lnet/minecraft/client/sound/SoundInstance;)V", cancellable = true, locals = LocalCapture.CAPTURE_FAILHARD)
    private void play(SoundInstance sound, CallbackInfo ci, WeightedSoundSet weightedSoundSet, Identifier identifier, Sound sound2, float f, float g, SoundCategory soundCategory, float h, float i, SoundInstance.AttenuationType attenuationType, boolean bl, Vec3d vec3d, boolean bl2, boolean bl3, CompletableFuture completableFuture, Channel.SourceManager sourceManager) {
        SoundMod.LOGGER.info("play() called");
        this.soundLoader.loadStreamed(sound2.getLocation(), false).thenAccept(stream -> sourceManager.run(source -> {
            source.setStream(stream);
            source.play();
        }));
        ci.cancel();
    }
}
