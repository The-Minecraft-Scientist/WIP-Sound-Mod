package net.randomscientist.soundmod.rust;

import net.minecraft.client.sound.AudioStream;

import java.nio.ByteBuffer;

import static net.randomscientist.soundmod.rust.SoundModLib.loadNatives;

public class SoundModNative {
    static {
        loadNatives();
    }
    native public static void say_hi();
    native public static void init(Class providerClass);
    native public static void get_sound_data(String path);
    native public static int new_sound_uuid();
    native public static void close_uuid(int id);
    native public static void play_uuid(int id);
    native public static void pause_uuid(int id);
    native public static void resume_uuid(int id);
    native public static void stop_uuid(int id);
    native public static boolean is_playing_uuid(int id);
    native public static boolean is_stopped_uuid( int id);
    native public static void set_position_uuid(int id, double x, double y, double z);
    native public static void set_pitch_uuid(int id, float pitch);
    native public static void set_looping_uuid(int id, boolean looping);
    native public static void set_relative_uuid(int id, boolean relative);
    native public static void set_ogg_stream_path_uuid(int id, String path);
    native public static void set_custom_stream_uuid(int id, AudioStream stream);
    native public static void run_debug_render();
    native public static void set_chunk(ByteBuffer buf, long chunk_x, long chunk_y);


}
