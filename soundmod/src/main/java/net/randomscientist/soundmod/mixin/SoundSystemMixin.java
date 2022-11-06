package net.randomscientist.soundmod.mixin;

import com.google.common.collect.Multimap;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;
import net.minecraft.SharedConstants;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.option.GameOptions;
import net.minecraft.client.sound.*;
import net.minecraft.resource.ResourceManager;
import net.minecraft.sound.SoundCategory;
import net.minecraft.util.Identifier;
import net.minecraft.util.math.Vec3d;
import net.randomscientist.soundmod.SoundMod;
import net.randomscientist.soundmod.natives.Natives;
import net.randomscientist.soundmod.util.ResourceDelegator;
import org.slf4j.Marker;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.lang.invoke.MethodHandle;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

import static net.minecraft.client.sound.SoundManager.MISSING_SOUND;
import static net.randomscientist.soundmod.natives.Natives.sender;
import static net.randomscientist.soundmod.util.ResourceDelegator.*;


@Mixin(SoundSystem.class)
public abstract class SoundSystemMixin {
    @Shadow protected abstract float getAdjustedPitch(SoundInstance sound);

    @Shadow protected abstract float getAdjustedVolume(float volume, SoundCategory category);

    @Shadow @Final private SoundManager loader;
    @Shadow @Final private List<SoundInstanceListener> listeners;
    @Shadow @Final private SoundListener listener;
    @Shadow @Final private Map<SoundInstance, Integer> soundEndTicks;
    @Shadow @Final private Map<SoundInstance, Channel.SourceManager> sources;
    @Shadow @Final private Multimap<SoundCategory, SoundInstance> sounds;
    @Shadow @Final private List<TickableSoundInstance> tickingSounds;
	@Shadow @Final private Channel channel;
	@Shadow private int ticks;

    @Shadow(aliases="shouldRepeatInstantly")
    private static boolean shouldRepeatInstantly(SoundInstance sound) {return false;}

    private ResourceManager resourceManager;
    @Inject(at = @At("TAIL"), method = "<init>")
    public void SoundSystem(SoundManager loader, GameOptions settings, ResourceManager resourceManager, CallbackInfo ci) {
        this.resourceManager = resourceManager;
    }
    private HashMap<SoundInstance, Long> sources2 = new HashMap<>();
    private int counter;

    /**
     * @author The-Minecraft-Scientist
     * @reason Rewrite sound backend
     **/

    @Overwrite(aliases = "play")
    /*public void play(SoundInstance sound) {
        sound.getSoundSet(MinecraftClient.getInstance().getSoundManager());
        if (sound.canPlay() && sound.getSound() != MISSING_SOUND) {
            Sound sound2 = sound.getSound();
            long uuid = ResourceDelegator.addResource(resourceManager, sound2.getLocation());
            MethodHandle addSound = Natives.getNativeHandle("add_sound");
            MemorySegment soundStruct = createSoundStruct(uuid,sound,ResourceDelegator.getSize(uuid));
            try {
                sender = (MemoryAddress) addSound.invoke(sender,soundStruct);
            } catch (Throwable e) {
                throw new RuntimeException(e);
            }
        }
    }*/
    public void play(SoundInstance sound) {
        if (sound.canPlay()) {
            WeightedSoundSet weightedSoundSet = sound.getSoundSet(this.loader);
            Identifier identifier = sound.getId();
            if (weightedSoundSet != null) {
                Sound sound2 = sound.getSound();
                if (sound2 != SoundManager.MISSING_SOUND) {
                    float f = sound.getVolume();
                    float g = Math.max(f, 1.0F) * (float)sound2.getAttenuation();
                    SoundCategory soundCategory = sound.getCategory();
                    float h = this.getAdjustedVolume(f, soundCategory);
                    float i = this.getAdjustedPitch(sound);
                    SoundInstance.AttenuationType attenuationType = sound.getAttenuationType();
                    boolean bl = sound.isRelative();
                    if(!(h == 0.0F && !sound.shouldAlwaysPlay())) {
                        Vec3d vec3d = new Vec3d(sound.getX(), sound.getY(), sound.getZ());
                        boolean bl2;
                        if (!this.listeners.isEmpty()) {
                            bl2 = bl || attenuationType == SoundInstance.AttenuationType.NONE || this.listener.getPos().squaredDistanceTo(vec3d) < (double)(g * g);
                            if (bl2) {
                                listeners.forEach((SoundInstanceListener listener) -> {
                                    listener.onSoundPlayed(sound,weightedSoundSet);
                                });
                            }
                        }

                        if (this.listener.getVolume() > 0.0F) {
                            this.counter++;
                            long uuid = sound2.hashCode() | (long) this.counter << 32;
                            bl2 = shouldRepeatInstantly(sound);
                            boolean bl3 = sound2.isStreamed();
                            SoundMod.LOGGER.info("Playing sound {} for event {}", sound2.getIdentifier(), identifier);
                            this.soundEndTicks.put(sound, this.ticks + 20);
                            this.sources2.put(sound, uuid);
                            this.sounds.put(soundCategory, sound);
                            // left as a reference
                            /* sourceManager.run((source) -> {
                                source.setPitch(i);
                                source.setVolume(h);
                                if (attenuationType == SoundInstance.AttenuationType.LINEAR) {
                                    source.setAttenuation(g);
                                } else {
                                    source.disableAttenuation();
                                }

                                source.setLooping(finalBl && !bl3);
                                source.setPosition(vec3d);
                                source.setRelative(bl);
                            });*/
                            //common loader logic
                            addResource(this.resourceManager,sound2.getLocation(), uuid);
                            int size = getSize(uuid);
                            MemorySegment struct =  createSoundStruct(uuid, sound, size);
                            if (!bl3) {
                                //static loader logic
                                byte[] buf = new byte[size];
                                if (readToJavaArray(uuid,buf) == size) {
                                    MemorySegment bufmem = allocBytes(buf);
                                    tryLoadStatic(struct, bufmem.address(), bufmem.byteSize());
                                }
                            } else {
                                //streaming loader logic
                                tryLoadStreaming(struct);
                            }

                            if (sound instanceof TickableSoundInstance) {
                                this.tickingSounds.add((TickableSoundInstance)sound);
                            }

                        }
                    }
                }
            }
        }
    }
}
