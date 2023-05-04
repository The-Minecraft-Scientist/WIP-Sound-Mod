package net.randomscientist.soundmod.rust;

import net.minecraft.client.sound.AudioStream;
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
    public static native boolean is_playing_uuid(int id);
    public static native boolean is_stopped_uuid( int id);
    public static native void set_position_uuid(int id, double x, double y, double z);
    public static native void set_pitch_uuid(int id, float pitch);
    public static native void set_looping_uuid(int id, boolean looping);
    public static native void set_relative_uuid(int id, boolean relative);
    public static native void set_ogg_stream_path_uuid(int id, String path);
    public static native void set_custom_stream_uuid(int id, AudioStream stream);


}
