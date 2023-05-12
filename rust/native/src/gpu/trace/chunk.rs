use bytemuck::{Pod, Zeroable};
use glam::{UVec2, UVec3, Vec3Swizzles};
const WORLD_SIZE: usize = 7;

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Material {
    some_val: f32,
}
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Chunk {
    // 16 chunk side * 16 chunk side * 384 world height * 16 bits / 32 bits of storage
    chunk_mrefs: [u32; 49152],
}
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct ChunkIndexTable {
    data: [[u32; WORLD_SIZE]; WORLD_SIZE],
}
impl ChunkIndexTable {
    fn index(&self, pos: UVec3) -> u32 {
        let cpos: UVec2 = pos.xy() >> 4;
        self.data[cpos.x as usize][cpos.y as usize]
    }
}
