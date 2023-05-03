package net.randomscientist.soundmod.mixins;

import net.minecraft.client.sound.AudioStream;
import net.minecraft.client.sound.Source;
import net.minecraft.client.sound.StaticSound;
import net.minecraft.util.math.Vec3d;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.gen.Invoker;

// Committing crimes with both indirection and magnitude
@Mixin(Source.class)
public class SourceMixin {
    @Final
    @Shadow
    private final int pointer = 0;

    @Invoker("<init>")
    static private Source newSource(int pointer) {
        throw new AssertionError();
    }

    @Overwrite
    static @Nullable Source create() {
        return newSource(SoundModNative.new_sound_uuid());
    }
    // Stop playback, close stream, free buffers, delete sources... etc.
    @Overwrite
    public void close() {}
    //Start playback (initial)
    @Overwrite
    public void play() {}
    //temporarily pause playback (without freeing/closing backing resources)
    @Overwrite
    public void pause() {}
    // If paused, resume playback. Else, no-op
    @Overwrite
    public void resume() {}
    // I genuinely do not know the difference between AL_STOPPED and AL_PAUSED, but this sets it to STOPPED
    @Overwrite
    public void stop() {}
    // whether this source is currently playing.
    @Overwrite
    public boolean isPlaying() {return true;}
    //Again, openAL shenanigans
    @Overwrite
    public boolean isStopped() {return false;}
    //Sets source position in world space (set_relative = false) or listener-relative space (set_relative = true)
    @Overwrite
    public void setPosition(Vec3d pos) {}
    //Sets source pitch (TODO: check openAL docs on how this should behave)
    @Overwrite
    public void setPitch(float pitch) {}
    //Whether the sound should loop over its buffer (pretty dang sure this should only work for static sounds)
    @Overwrite
    public void setLooping(boolean looping) {}
    //This is a no-op, we're doing some Shenanigans™
    @Overwrite
    public void disableAttenuation() {}
    //This is a no-op, we're doing some Shenanigans™
    @Overwrite
    public void setAttenuation(float attenuation) {}
    // False: sound position is in global, world space coordinates, True: sound position is in local, listener-relative coordinates.
    @Overwrite
    public void setRelative(boolean relative) {}
    //Need to do some jankery here
    @Overwrite
    public void setBuffer(StaticSound sound) {}
    //More jankery :D
    @Overwrite
    public void setStream(AudioStream stream) {}
    //This can probably be a no-op. Nice to have though.
    @Overwrite
    public void tick() {}
}
