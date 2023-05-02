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
        return newSource(0);
    }
    @Overwrite
    public void close() {}
    @Overwrite
    public void play() {}
    @Overwrite
    public void pause() {}

    @Overwrite
    public void resume() {}

    @Overwrite
    public void stop() {}

    @Overwrite
    public boolean isPlaying() {return true;}

    @Overwrite
    public boolean isStopped() {return false;}

    @Overwrite
    public void setPosition(Vec3d pos) {}

    @Overwrite
    public void setPitch(float pitch) {}

    @Overwrite
    public void setLooping(boolean looping) {}

    @Overwrite
    public void disableAttenuation() {}

    @Overwrite
    public void setAttenuation(float attenuation) {}

    @Overwrite
    public void setRelative(boolean relative) {}

    @Overwrite
    public void setBuffer(StaticSound sound) {}

    @Overwrite
    public void setStream(AudioStream stream) {}

    @Overwrite
    public void tick() {}
}
