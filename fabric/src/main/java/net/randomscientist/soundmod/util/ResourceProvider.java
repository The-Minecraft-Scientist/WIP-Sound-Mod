package net.randomscientist.soundmod.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourceFactory;
import net.minecraft.resource.ResourceManager;

public class ResourceProvider {
    static ResourceManager manager;
    static {
        //This depends on a MinecraftClient instance being available at class load-time.
        // Might end up getting moved to init code elsewhere
        manager = MinecraftClient.getInstance().getResourceManager();
    }
    public static void registerInputStream(String name) {
        try {

        } catch {

        }
    }
}
