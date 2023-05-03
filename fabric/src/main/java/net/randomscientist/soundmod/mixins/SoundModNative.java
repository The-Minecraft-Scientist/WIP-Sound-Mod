package net.randomscientist.soundmod.mixins;

import net.minecraft.util.math.Vec3d;

public class SoundModNative {
    public static native void say_hi();
    public static native void init(Class providerClass);
    public static native void get_sound_data(String path);
    public static native int new_sound_uuid();
    public static native void close_uuid(int id);
    public static native void play_uuid(int id);
    public static native void pause_uuid(int id);
    public static native void resume_uuid(int id);
    public static native void stop_uuid(int id);
    public static native void is_playing_uuid(int id);
    public static native void is_stopped_uuid( int id);
    public static native void set_position_uuid(int id, Vec3d pos);
    public static native void set_pitch_uuid(int id, float pitch);
    public static native void set_looping_uuid(int id, boolean looping);
    public static native void set_relative_uuid(int id, boolean relative);


}
