use bytemuck::{Pod, Zeroable};
use glam::{IVec2, UVec2, UVec4};

#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Material {
    some_val: f32,
}
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Chunk {
    // 16 chunk side * 16 chunk side * 384 world height * 16 bits / 32 bits of storage per u32
    chunk_mrefs: [u32; 49152],
}
impl Chunk {
    pub const SINGLE_CHUNK_MREF_BUF_BYTE_SIZE: u32 = 16 * 16 * 384 * 2;
    pub const SINGLE_SECTION_MREF_BUF_BYTE_SIZE: u32 = Self::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE / 24;
}
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct ChunkIndexTable {
    data: [UVec4; 256],
    radius: u32,
}
impl ChunkIndexTable {
    pub const fn new(radius: u32) -> Self {
        Self {
            data: [UVec4::splat(0); 256],
            radius,
        }
    }
    /// Sets the chunk offset corresponding to this position
    pub fn set_at(&mut self, mut pos: IVec2, offset: u32) {
        pos += IVec2::splat(self.radius as i32);
        assert!(pos.cmpge(IVec2::splat(0)).all());
        let pos = pos.as_uvec2();
        assert!(pos.x < 32 || pos.y < 32);
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
    pub fn get_at(&mut self, pos: UVec2) -> u32 {
        assert!(pos.x < 32 || pos.y < 32);
        let index = (pos.y << 4) | pos.x;
        let mut val: &mut UVec4 = &mut self.data[index as usize];
        match index & 3 {
            0 => val.x,
            1 => val.y,
            2 => val.z,
            3 => val.w,
            _ => {
                panic!("unreachable code")
            }
        }
    }
}
pub struct ChunkIndexTransform {
    origin: IVec2,
    radius: u32,
}
impl ChunkIndexTransform {
    pub fn transform_to_local(&self, global_chunk_pos: IVec2) -> Option<UVec2> {
        let mut res = global_chunk_pos - self.origin;
        res += IVec2::splat(self.radius as i32);
        return Some(res.as_uvec2());
    }
}
