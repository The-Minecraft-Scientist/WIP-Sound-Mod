use crate::gpu::trace::chunk::{Chunk, ChunkIndexTable, Material};
use crate::gpu::trace::uniforms::Uniforms;
use glam::{I64Vec2, IVec2, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::StagingBelt;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferSize, CommandEncoder, Device};
#[derive(Copy, Clone, Debug, Default)]
pub struct RunningWorldState {
    pub center_chunk: I64Vec2,
    pub client_player_pos: Vec3,
    pub client_player_look_dir: Vec3,
}
pub struct TraceState {
    chunk_buffer: Buffer,
    material_buf: Buffer,
    pub uniforms: Uniforms,
    pub staging_belt: StagingBelt,
    chunk_allocator: ChunkAllocator,
    current_diff: Option<Vec<TraceStateChange>>,
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
            size: (Material::SIZE as usize * (u16::MAX as usize)) as BufferAddress,
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
            uniforms: Uniforms::default(),
        }
    }
    pub fn queue_diff(&mut self, change: TraceStateChange) {
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
        if self.current_diff.len() == 0 {
            self.staging_belt.finish();
            return;
        }
        for diff in self.current_diff.iter() {
            match diff {
                TraceStateChange::Section { location, new } => {
                    let mut view = self.staging_belt.write_buffer(
                        encoder,
                        &self.chunk_buffer,
                        self.chunk_allocator.get_or_alloc(location.chunk_coords)
                            + location.section_index as BufferAddress
                                * Chunk::SINGLE_SECTION_MREF_BUF_BYTE_SIZE as BufferAddress,
                        BufferSize::new(Chunk::SINGLE_SECTION_MREF_BUF_BYTE_SIZE as u64).unwrap(),
                        device,
                    );
                    view.copy_from_slice(bytemuck::cast_slice(new.as_slice()));
                }
                WorldChange::Material { id, new } => {
                    let mut view = self.staging_belt.write_buffer(
                        encoder,
                        &self.material_buf,
                        *id as BufferAddress * Material::SIZE,
                        BufferSize::new(Material::SIZE).unwrap(),
                        device,
                    );
                    view.copy_from_slice(bytemuck::cast_ref::<
                        Material,
                        [u8; Material::SIZE as usize],
                    >(new))
                }
                WorldChange::WorldChunkCenter { new } => {
                    self.running_world_state.center_chunk = *new;
                }
                WorldChange::PlayerInfo { pos, look_dir } => {
                    self.running_world_state.client_player_look_dir = *look_dir;
                    self.running_world_state.client_player_pos = *pos;
                }
                TraceStateChange::Material { id, new } => {
                    self.staging_belt.write_buffer(encoder, self.material_buf)
                }
            }
        }
        self.current_diff.clear();
        println!("finishing staging belt");
        self.staging_belt.finish();
    }

    pub fn make_chunk_index_table(&self, center: IVec2) -> ChunkIndexTable {
        let mut table = ChunkIndexTable::new(3);
        for entry in self.chunk_allocator.chunks.iter() {
            if let Some(diff) = self.contains(*entry.0) {
                table.set_at(
                    diff,
                    (entry.1.location / Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as u64) as u32,
                )
            }
        }
        table
    }
    pub fn contains(&self, a: I64Vec2) -> Option<IVec2> {
        let diff = (a - self.running_world_state.center_chunk).as_ivec2();
        if !(diff.x > ((self.radius) as i32 + 1)
            || diff.x < -(self.radius as i32)
            || diff.y > (self.radius) as i32 + 1
            || diff.y < -(self.radius as i32))
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
pub struct ChunkAllocation {
    location: BufferAddress,
    id: usize,
}

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
        if self.chunks.len()
            == (self.buffer_size / Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as usize - 1)
        {
            let oldest = {
                let mut iter = self.chunks.iter();
                let mut oldest = iter.next().unwrap();
                //drop the oldest chunk
                for i in iter {
                    if i.1.id < oldest.1.id {
                        oldest = i;
                    }
                }
                (*oldest.0, *oldest.1)
            };
            let _ = self.chunks.remove(&oldest.0);
            let _ = self.chunks.insert(
                chunk_coords,
                ChunkAllocation {
                    location: oldest.1.location,
                    id: self.counter,
                },
            );
            self.counter += 1;
            return oldest.1.location;
        }
        let head = match self.current_head {
            Some(h) => h,
            None => ChunkAllocation { id: 0, location: 0 },
        };
        let new_index = head.location + Chunk::SINGLE_CHUNK_MREF_BUF_BYTE_SIZE as BufferAddress;
        let new_head = ChunkAllocation {
            id: self.counter,
            location: new_index,
        };
        self.counter += 1;
        self.current_head = Some(new_head);
        let _ = self.chunks.insert(chunk_coords, new_head);
        new_index
    }
    pub fn get_or_alloc(&mut self, chunk_coords: I64Vec2) -> BufferAddress {
        if let Some(alloc) = self.chunks.get(&chunk_coords) {
            return alloc.location;
        };
        self.allocate(chunk_coords)
    }
}

pub struct WorldStateDiff(Vec<TraceStateChange>);
#[derive(Clone, Debug)]
pub enum TraceStateChange {
    Section {
        location: ChunkSectionLocation,
        new: Arc<[u16; 16 * 16 * 16]>,
    },
    Material {
        id: u16,
        new: Material,
    },
    Player {
        position: Vec3,
        look_dir: Vec3,
    },
}
#[derive(Copy, Clone, Debug)]
pub struct ChunkSectionLocation {
    pub chunk_coords: I64Vec2,
    pub section_index: u16,
}
