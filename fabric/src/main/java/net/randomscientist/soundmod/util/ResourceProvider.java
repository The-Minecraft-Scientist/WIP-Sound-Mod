package net.randomscientist.soundmod.util;

import net.minecraft.client.MinecraftClient;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;

import java.io.InputStream;

public class ResourceProvider {
    public static ResourceManager manager = null;

    @SuppressWarnings("unused")
    public static InputStream getResourceStream(String id) {
        if(manager == null) {
            manager = MinecraftClient.getInstance().getResourceManager();
        }
        Identifier ident = new Identifier(id);
        try {
            return manager.open(ident);
        } catch(Exception e) {
            throw new RuntimeException(e);
        }
    }
}

