package net.randomscientist.soundmod.mixin;

import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.sound.*;
import net.minecraft.resource.ResourceManager;
import net.randomscientist.soundmod.natives.Natives;
import net.randomscientist.soundmod.util.ResourceDelegator;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.lang.invoke.MethodHandle;



@Mixin(SoundSystem.class)
public abstract class SoundSystemMixin {
    private ResourceManager resourceManager;
    @Inject(at = @At("TAIL"), method = "<init>")
    public void SoundSystem(SoundManager loader, GameOptions settings, ResourceManager resourceManager, CallbackInfo ci) {
        this.resourceManager = resourceManager;
    }
    /**
     * @author The-Minecraft-Scientist
     * @reason Rewrite sound backend
     */
    @Overwrite(aliases = "play")
    public void play(SoundInstance sound) {
        if (sound.canPlay()) {
            sound.getSoundSet(MinecraftClient.getInstance().getSoundManager());
            Sound sound2 = sound.getSound();
            long uuid = ResourceDelegator.addResource(resourceManager, sound2.getLocation());
            MethodHandle playInputStream = Natives.getNativeHandle("play_input_stream");
            try {
                playInputStream.invoke(uuid,Natives.getMethodSymbol("readStream").address(),Natives.getMethodSymbol("seekStream").address(),ResourceDelegator.getSize(uuid));
            } catch (Throwable e) {
                throw new RuntimeException(e);
            }
        }
    }
}
