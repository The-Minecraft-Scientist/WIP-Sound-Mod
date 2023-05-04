package net.randomscientist.soundmod.sound;

import net.minecraft.client.sound.AudioStream;
import net.minecraft.util.Identifier;

public interface ResourcePathAudioStream extends AudioStream {
    Identifier get_path();
}
