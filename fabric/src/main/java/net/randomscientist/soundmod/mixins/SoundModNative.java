package net.randomscientist.soundmod.mixins;

public class SoundModNative {
    public static native void say_hi();
    public static native void init(Class providerClass);
    public static native void get_sound_data(String id);
    public static native int new_sound_uuid();
}
