use crate::gpu::trace::chunk::{Chunk, ChunkIndexTable, Material};
use glam::IVec3;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, Device};

pub struct SoundModTraceState<'a> {
    chunk_buffer: Buffer,
    material_buf: Buffer,
    offset_table: ChunkIndexTable,
    current_diff: Option<Vec<WorldChange<'a>>>,
}
pub const AUDIO_WORLD_SIDE: u32 = 16;

impl<'a> SoundModTraceState<'a> {
    pub fn new(device: &mut Device) -> Self {
        let chunk_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Chunk buffer"),
            size: (AUDIO_WORLD_SIDE * AUDIO_WORLD_SIDE * Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE)
                as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let material_buf = device.create_buffer(&BufferDescriptor {
            label: Some("Material buffer"),
            size: (std::mem::size_of::<Material>() * (u16::MAX as usize)) as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let offset_table = ChunkIndexTable::new();
        Self {
            chunk_buffer,
            material_buf,
            offset_table,
            current_diff: vec![],
        }
    }
    pub fn queue_diff(&mut self, change: WorldChange<'a>) {
        if let Some(diff) = &mut self.current_diff {
            diff.push(change);
            return;
        }
        self.current_diff = Some(vec![]);
        //SAFETY: we have ensured that current diff is Some
        unsafe { &mut self.current_diff.unwrap_unchecked().push(change) };
    }
    pub fn apply_diffs(&mut self) {
        let Some(diffs) = &mut self.current_diff else {
            return;
        };
    }
}

pub struct WorldStateDiff<'a>(Vec<WorldChange<'a>>);
#[derive(Clone, Debug)]
pub enum WorldChange<'a> {
    SingleBlock {
        pos: IVec3,
        new_mat: u16,
    },
    Section {
        location: ChunkSectionLocation,
        new: &'a [u16; Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as usize],
    },
    Material {
        id: u16,
        new: Material,
    },
}
#[derive(Copy, Clone, Debug)]
pub struct ChunkSectionLocation {
    chunk_x: i32,
    chunk_z: i32,
    section_index: u16,
}
