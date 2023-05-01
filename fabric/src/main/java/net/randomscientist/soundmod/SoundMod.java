package net.randomscientist.soundmod;

import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.player.UseItemCallback;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.sound.PositionedSoundInstance;
import net.minecraft.client.sound.Sound;
import net.minecraft.client.sound.SoundInstance;
import net.minecraft.client.sound.WeightedSoundSet;
import net.minecraft.resource.Resource;
import net.minecraft.resource.ResourceManager;
import net.minecraft.sound.SoundCategory;
import net.minecraft.sound.SoundEvents;
import net.minecraft.util.ActionResult;
import net.minecraft.util.Identifier;
import net.minecraft.util.TypedActionResult;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.random.Random;
import net.minecraft.util.math.random.RandomSplitter;
import net.randomscientist.soundmod.rust.SoundModNative;
import net.randomscientist.soundmod.util.ResourceProvider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.InputStream;

public class SoundMod implements ModInitializer {
    // This logger is used to write text to the console and the log file.
    // It is considered best practice to use your mod id as the logger's name.
    // That way, it's clear which mod wrote info, warnings, and errors.
    public static final Logger LOGGER = LoggerFactory.getLogger("soundmod");
    @Override
    public void onInitialize() {
        // This code runs as soon as Minecraft is in a mod-load-ready state.
        // However, some things (like resources) may still be uninitialized.
        // Proceed with mild caution.
        SoundModNative.say_hi();
        Class c;
        try {
            c = this.getClass().getClassLoader().loadClass("net.randomscientist.soundmod.util.ResourceProvider");
        } catch (ClassNotFoundException e) {
            throw new RuntimeException(e);
        }
        SoundModNative.init(c);
        SoundInstance ins = new PositionedSoundInstance(SoundEvents.BLOCK_AMETHYST_BLOCK_BREAK, SoundCategory.AMBIENT, 1.0f, 1.0f, Random.create(), new BlockPos(1, 1, 1));

        UseItemCallback.EVENT.register((player, world, hand) -> {
            WeightedSoundSet set = ins.getSoundSet(MinecraftClient.getInstance().getSoundManager());
            Sound sound2 = ins.getSound();
            String thing = sound2.getLocation().toString();
            SoundModNative.get_sound_data(thing);
            return TypedActionResult.pass(player.getMainHandStack());
        });
        LOGGER.info("Hello Fabric world!");
    }
}