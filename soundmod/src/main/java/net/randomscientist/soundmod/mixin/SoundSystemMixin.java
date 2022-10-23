package net.randomscientist.soundmod.mixin;

import net.minecraft.client.sound.SoundInstance;
import net.minecraft.client.sound.SoundSystem;
import net.randomscientist.soundmod.SoundMod;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(SoundSystem.class)
public class SoundSystemMixin {

    /**
     * @author
     * The-Minecraft-Scientist
     * @reason
     * Rewrite sound backend
     */
    @Overwrite(aliases = "play")
    public void play(SoundInstance sound) {
        SoundMod.LOGGER.info(String.valueOf(sound));
    }
}
