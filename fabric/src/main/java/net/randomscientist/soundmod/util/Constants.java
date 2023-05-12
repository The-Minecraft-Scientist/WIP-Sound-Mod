package net.randomscientist.soundmod.util;

import java.util.Arrays;

public class Constants {
    public static final int CHUNK_MREF_BUF_BYTE_SIZE = 16 * 16 * 384 * 2;
    public static final short[] EMPTY_CHUNK_SECTION;
    static {
        EMPTY_CHUNK_SECTION = new short[4096];
        Arrays.fill(EMPTY_CHUNK_SECTION, (short)0);
    }
}
