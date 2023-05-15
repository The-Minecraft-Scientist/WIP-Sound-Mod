use bytemuck::{Pod, Zeroable};
use glam::{UVec2, UVec4, Vec3Swizzles};
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
    data: [UVec4; 256],
}
impl ChunkIndexTable {
    pub const fn new() -> Self {
        Self {
            data: [UVec4::splat(0); 256],
        }
    }
    /// Sets the chunk offset corresponding to this position
    pub fn set_at_unchecked(&mut self, pos: UVec2, offset: u32) {
        let index = (pos.y << 4) | pos.x;
        let mut val: &mut UVec4 = &mut self.data[index as usize];
        match index & 3 {
            0 => val.x = offset,
            1 => val.y = offset,
            2 => val.z = offset,
            3 => val.w = offset,
            _ => {
                panic!("unreachable code")
            }
        }
    }
    fn set_at(&mut self, pos: UVec2, offset: u32) {
        self.set_at_unchecked()
    }
}
