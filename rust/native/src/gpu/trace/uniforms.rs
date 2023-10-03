use bytemuck::{Pod, Zeroable};
use glam::{UVec4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub chunk_index_table: [UVec4; 256],
    pub player_position: Vec3,
    pub player_look_dir: Vec3,
}
impl Default for Uniforms {
    fn default() -> Self {
        Self {
            chunk_index_table: [UVec4::ZERO; 256],
            player_position: Vec3::ZERO,
            player_look_dir: Vec3::ZERO,
        }
    }
}
