package net.randomscientist.soundmod.scene;

import net.minecraft.client.world.ClientChunkManager;
import net.minecraft.entity.player.PlayerEntity;
import net.minecraft.util.math.Vec3i;
import net.minecraft.world.chunk.WorldChunk;
import net.randomscientist.soundmod.util.Constants;
import net.randomscientist.soundmod.util.Pos2i;
import net.randomscientist.soundmod.util.WorldChunkAccessor;

import java.util.ArrayList;
import java.util.Arrays;

public class Scene {
    final ClientChunkManager cm;
    final PlayerEntity player;
    private int size = 4;
    Pos2i playerChunk = new Pos2i(0, 0);
    public int[] data;
    public Scene(ClientChunkManager cm, PlayerEntity player, int size) {
        this.cm = cm;
        this.player = player;
        this.size = size;
    }
    public Pos2i getPlayerChunk() {
        playerChunk.x = player.getBlockX()>>4;
        playerChunk.y = player.getBlockZ()>>4;
        return playerChunk;
    }

    public void fullScene() {
        getPlayerChunk();
        ArrayList<Integer> data = new ArrayList<>();
        for(int i = playerChunk.x-size; i <=playerChunk.x+size; i++) {
            for(int j = playerChunk.y-size; j <= playerChunk.y+size; j++) {
                WorldChunk chunk = (WorldChunk) cm.getChunk(i,j);
                if(chunk != null) {
                    data.addAll(((WorldChunkAccessor) chunk).getSoundScene());
                }
                else {
                    data.addAll(Constants.EMPTY_CHUNK);
                }
            }
        }
        this.data = data.stream().mapToInt(i -> i).toArray();
    }
    public Vec3i worldToScene(Vec3i pos) {
        Vec3i baseChunkWorldCoordinate = new Vec3i((playerChunk.x-size)<<4,-60,(playerChunk.y-size)<<4);
        return baseChunkWorldCoordinate.subtract(pos);
    }
    public int posToIndex(Vec3i lpos) {
        return (lpos.getX() >> 4 + lpos.getZ() >> 4 * size) * 81920 + ((lpos.getY() << 4 | (lpos.getZ() & 0xF)) << 4 | (lpos.getX() & 0xF));
    }
    public int readIndex(int index) {
        return data[index>>5]&(0x1<<(index&0x1F));
    }
}
