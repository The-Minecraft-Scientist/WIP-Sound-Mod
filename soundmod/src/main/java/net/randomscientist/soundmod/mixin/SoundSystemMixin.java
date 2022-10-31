package net.randomscientist.soundmod.mixin;

import jdk.incubator.foreign.*;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.sound.*;
import net.minecraft.resource.ResourceManager;
import net.randomscientist.soundmod.SoundMod;
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
            /*
            if(!this.listeners.isEmpty()) {
                WeightedSoundSet weightedSoundSet = sound.getSoundSet(this.loader);
                Vec3d vec3d = new Vec3d(sound.getX(), sound.getY(), sound.getZ());
                float g = Math.max(sound.getVolume(), 1.0F) * (float)sound2.getAttenuation();
                boolean bl = sound.isRelative();
                boolean bl2 = bl || sound.getAttenuationType() == SoundInstance.AttenuationType.NONE || this.listener.getPos().squaredDistanceTo(vec3d) < (double)(g * g);
                if (bl2) {
                    Iterator var14 = this.listeners.iterator();
                    while(var14.hasNext()) {
                        SoundInstanceListener soundInstanceListener = (SoundInstanceListener)var14.next();
                        soundInstanceListener.onSoundPlayed(sound, weightedSoundSet);
                    }
                }*/
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
