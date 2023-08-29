use bytemuck::{Pod, Zeroable};
use glam::{UVec4, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    chunk_index_table: [UVec4; 256],
    player_position: Vec3,
    player_look_dir: Vec3,
}
