package net.randomscientist.soundmod.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import net.randomscientist.soundmod.SoundMod;

import java.io.InputStream;

public class ResourceProvider {
    static ResourceManager manager;
    static {
        //This depends on a MinecraftClient instance being available at class load-time.
        // Might end up getting moved to init code elsewhere
        manager = MinecraftClient.getInstance().getResourceManager();
    }
    @SuppressWarnings("unused")
    public static InputStream getResourceStream(String id) {
        Identifier ident = new Identifier(id);
        try {
            return manager.open(ident);
        } catch(Exception e) {
            throw new RuntimeException(e);
        }
    }
}
