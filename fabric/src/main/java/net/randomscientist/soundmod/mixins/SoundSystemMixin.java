package net.randomscientist.soundmod.mixins;

import net.minecraft.client.sound.*;
import net.minecraft.sound.SoundCategory;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.Vec3d;
import org.spongepowered.asm.mixin.*;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

import java.util.List;
import java.util.concurrent.CompletableFuture;
@Mixin(SoundSystem.class)
public class SoundSystemMixin {
    @Shadow
    @Final
    private SoundLoader soundLoader;

    @Shadow
    @Final
    private List<TickableSoundInstance> tickingSounds;


    @Inject(at = @At(value = "INVOKE", target = "Lnet/minecraft/client/sound/Channel$SourceManager;run(Ljava/util/function/Consumer;)V"), method = "play(Lnet/minecraft/client/sound/SoundInstance;)V", cancellable = true, locals = LocalCapture.CAPTURE_FAILHARD)
    private void play(SoundInstance sound, CallbackInfo ci, WeightedSoundSet _sound_soundset, Identifier sound_identifier, Sound built_sound, float f, float g, SoundCategory category, float h, float i, SoundInstance.AttenuationType attenuationType, boolean bl, Vec3d vec3d, boolean bl2, boolean bl3, CompletableFuture completableFuture, Channel.SourceManager sourceManager) {
        this.soundLoader.loadStreamed(built_sound.getLocation(), false).thenAccept(stream -> sourceManager.run(source -> {
            source.setStream(stream);
            source.play();
        }));
        if (sound instanceof TickableSoundInstance) {
            this.tickingSounds.add((TickableSoundInstance)sound);
        }
        ci.cancel();
    }
}
