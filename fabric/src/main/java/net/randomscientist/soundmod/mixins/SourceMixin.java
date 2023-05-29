package net.randomscientist.soundmod.mixins;

import net.minecraft.client.sound.AudioStream;
import net.minecraft.client.sound.Source;
import net.minecraft.client.sound.StaticSound;
import net.minecraft.util.math.Vec3d;
import net.randomscientist.soundmod.SoundModClient;
import net.randomscientist.soundmod.rust.SoundModNative;
import net.randomscientist.soundmod.sound.ResourcePathAudioStream;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.gen.Invoker;
import static net.randomscientist.soundmod.rust.SoundModNative.*;
@Mixin(Source.class)
public class SourceMixin {
    //We reuse pointer for crimes™
    @Final
    @Shadow
    private int pointer;

    @Invoker("<init>")
    static private Source newSource(final int ptr) {
        throw new AssertionError();
    }

    @Overwrite
    public static @Nullable Source create() {
        int i = new_sound_uuid();
        return newSource(i);
    }
    // Stop playback, close stream, free buffers, delete sources... etc.
    @Overwrite
    public void close() {close_uuid(this.pointer);}
    //Start playback (initial)
    @Overwrite
    public void play() {play_uuid(this.pointer);}
    //temporarily pause playback (without freeing/closing backing resources)
    @Overwrite
    public void pause() {pause_uuid(this.pointer);}
    // If paused, resume playback. Else, no-op
    @Overwrite
    public void resume() {resume_uuid(this.pointer);}
    // I genuinely do not know the difference between AL_STOPPED and AL_PAUSED, but this sets it to STOPPED
    @Overwrite
    public void stop() {stop_uuid(this.pointer);}
    // whether this source is currently playing.
    @Overwrite
    public boolean isPlaying() {return is_playing_uuid(this.pointer);}
    //Again, openAL shenanigans
    @Overwrite
    public boolean isStopped() {return is_stopped_uuid(this.pointer);}
    //Sets source position in world space (set_relative = false) or listener-relative space (set_relative = true)
    @Overwrite
    public void setPosition(Vec3d pos) {set_position_uuid(this.pointer, pos.x, pos.y, pos.z);}
    //Sets source pitch (TODO: check openAL docs on how this should behave)
    @Overwrite
    public void setPitch(float pitch) {set_pitch_uuid(this.pointer, pitch);}
    //Whether the sound should loop over its buffer (pretty dang sure this should only work for static sounds)
    @Overwrite
    public void setLooping(boolean looping) {set_looping_uuid(this.pointer, looping);}
    //This is a no-op, we're doing some Shenanigans™
    @Overwrite
    public void disableAttenuation() {}
    //This is a no-op, we're doing some Shenanigans™
    @Overwrite
    public void setAttenuation(float attenuation) {}
    // False: sound position is in global, world space coordinates, True: sound position is in local, listener-relative coordinates.
    @Overwrite
    public void setRelative(boolean relative) {set_relative_uuid(this.pointer, relative);}
    // This is a no-op. StaticSounds are heavily entangled with AL, I'm not gonna bother steamrolling them like what I did this class.
    @Overwrite
    public void setBuffer(StaticSound sound) {}
    //More jankery :D
    @Overwrite
    public void setStream(AudioStream stream) {
        if(stream instanceof ResourcePathAudioStream) {
            //This stream is a funny stream that can do funny things (like nothing)
            String name = ((ResourcePathAudioStream) stream).get_path().toString();
            SoundModClient.LOGGER.info("sound #" + this.pointer + " requesting id: " + name);
            SoundModNative.set_ogg_stream_path_uuid(this.pointer, name);
        } else {
            SoundModClient.LOGGER.info("normal audio stream called :((((");
            //Boring custom audio impl that will be a pain in the ass to support, so I won't (for now)
            SoundModNative.set_custom_stream_uuid(this.pointer, stream);
        }
    }
    //This can probably be a no-op. Nice to have though.
    @Overwrite
    public void tick() {}
}
