use crate::gpu::trace::chunk::{Chunk, ChunkIndexTable, Material};
use glam::{I64Vec2, IVec2};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::StagingBelt;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferSize, CommandEncoder, Device};

pub struct TraceState {
    chunk_buffer: Buffer,
    material_buf: Buffer,
    pub staging_belt: StagingBelt,
    chunk_allocator: ChunkAllocator,
    current_diff: Option<Vec<WorldChange>>,
    center_chunk: I64Vec2,
    world_radius: u32,
}
pub const AUDIO_WORLD_SIDE: u32 = 16;
pub const CHUNK_BUFFER_SIZE: u32 =
    (AUDIO_WORLD_SIDE * AUDIO_WORLD_SIDE * Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE * 2);
impl TraceState {
    pub fn new(device: &Device) -> Self {
        let chunk_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Chunk buffer"),
            size: CHUNK_BUFFER_SIZE as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let material_buf = device.create_buffer(&BufferDescriptor {
            label: Some("Material buffer"),
            size: (std::mem::size_of::<Material>() * (u16::MAX as usize)) as BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let staging_belt =
            StagingBelt::new(Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as BufferAddress);
        let chunk_allocator = ChunkAllocator::new(CHUNK_BUFFER_SIZE as usize);
        Self {
            chunk_buffer,
            material_buf,
            staging_belt,
            chunk_allocator,
            current_diff: Some(vec![]),
            center_chunk: I64Vec2::ZERO,
            world_radius: 3,
        }
    }
    pub fn queue_diff(&mut self, change: WorldChange) {
        if let Some(diff) = &mut self.current_diff {
            diff.push(change);
            return;
        }
        self.current_diff = Some(vec![]);
        //SAFETY: we have ensured that current diff is Some
        //TODO: this is stupid, llvm will probably just optimize the checks out
        unsafe { self.current_diff.as_mut().unwrap_unchecked().push(change) }
    }
    pub fn apply_diffs(&mut self, device: &Device, encoder: &mut CommandEncoder) {
        self.staging_belt.recall();
        let Some(diffs) = self.current_diff.take() else {
            self.staging_belt.finish();
            return;
        };
        for diff in diffs.into_iter() {
            match diff {
                WorldChange::Section { location, new } => {
                    let mut view = self.staging_belt.write_buffer(
                        encoder,
                        &self.chunk_buffer,
                        self.chunk_allocator.get_or_alloc(location.chunk_coords)
                            + location.section_index as BufferAddress
                                * Chunk::SINGLE_SECTION_MREF_BUF_BYTE_SIZE as BufferAddress,
                        BufferSize::new(Chunk::SINGLE_SECTION_MREF_BUF_BYTE_SIZE as u64).unwrap(),
                        device,
                    );
                    view.copy_from_slice(bytemuck::cast_slice(new.as_ref()));
                }
                WorldChange::Material { id, new } => {}
            }
        }
        println!("finishing staging belt");
        self.staging_belt.finish();
    }

    pub fn make_chunk_index_table(&self, center: IVec2) -> ChunkIndexTable {
        let mut table = ChunkIndexTable::new(3);
        for entry in self.chunk_allocator.chunks.iter() {
            if let Some(diff) = self.contains(*entry.0) {
                table.set_at(
                    diff,
                    (entry.1 .0 / Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as u64) as u32,
                )
            }
        }
        table
    }
    pub fn contains(&self, a: I64Vec2) -> Option<IVec2> {
        let diff = (a - self.center_chunk).as_ivec2();
        if !(diff.x > ((self.world_radius) as i32 + 1)
            || diff.x < -(self.world_radius as i32)
            || diff.y > (self.world_radius) as i32 + 1
            || diff.y < -(self.world_radius as i32))
        {
            Some(diff)
        } else {
            None
        }
    }
}

pub struct ChunkAllocator {
    pub chunks: HashMap<I64Vec2, ChunkAllocation>,
    counter: usize,
    buffer_size: usize,
    current_head: Option<ChunkAllocation>,
}
#[derive(Copy, Clone, Debug)]
pub struct ChunkAllocation(BufferAddress, usize);

impl ChunkAllocator {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            chunks: HashMap::with_capacity(128),
            buffer_size,
            counter: 0,
            current_head: None,
        }
    }
    pub fn allocate(&mut self, chunk_coords: I64Vec2) -> BufferAddress {
        //TODO: make this a bit more intelligent
        if self.chunks.len() == self.buffer_size / Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as usize {
            let oldest = {
                let mut iter = self.chunks.iter();
                let mut oldest = iter.next().unwrap();
                //drop the oldest chunk
                for i in iter {
                    if i.1 .1 < oldest.1 .1 {
                        oldest = i;
                    }
                }
                (*oldest.0, *oldest.1)
            };
            let _ = self
                .chunks
                .insert(chunk_coords, ChunkAllocation(oldest.1 .0, self.counter));
            self.counter += 1;
            return oldest.1 .0;
        }
        let head = match self.current_head {
            Some(h) => h,
            None => ChunkAllocation(0, 0),
        };
        let new_index = head.0 + Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as BufferAddress;
        self.counter += 1;
        let _ = self
            .chunks
            .insert(chunk_coords, ChunkAllocation(new_index, self.counter));
        new_index
    }
    pub fn get_or_alloc(&mut self, chunk_coords: I64Vec2) -> BufferAddress {
        println!("fetching chunk at {}", chunk_coords);
        if let Some(alloc) = self.chunks.get(&chunk_coords) {
            return alloc.0;
        };
        self.allocate(chunk_coords)
    }
}

pub struct WorldStateDiff(Vec<WorldChange>);
#[derive(Clone, Debug)]
pub enum WorldChange {
    Section {
        location: ChunkSectionLocation,
        new: Arc<[u16; 16 * 16 * 16]>,
    },
    Material {
        id: u16,
        new: Material,
    },
}
#[derive(Copy, Clone, Debug)]
pub struct ChunkSectionLocation {
    pub chunk_coords: I64Vec2,
    pub section_index: u16,
}
