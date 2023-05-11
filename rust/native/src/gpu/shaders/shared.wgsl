const RESC_WORLD_SIDE = 7u;
struct Material {
    property_idk: f32,
}
// 24576 = 16 * 16 * 384 / 4 (16x16x384 chunk, 1byte material refs, 4 bytes per u32)
struct Chunk {
    chunk_mask: array<u32, 24576>,
    // only store 255 materials, we can hardcode the properties of AIR (mat 0)
    mats: array<Material, 255>,
}

struct ChunkIndexTable {
    data: array<array<u32, RESC_WORLD_SIDE>, RESC_WORLD_SIDE>
}

@group(0) @binding(0)
var<storage, read> chunk_index_table: ChunkIndexTable;

@group(0) @binding(1)
var<storage, read> chunk_data: array<Chunk>;

fn block_mref(pos: vec3<u32>) -> bool {
    let cpos = pos.xz >> vec2(4u);
    let chunkptr = &chunk_data[(chunk_index_table.data[cpos.x][cpos.y])];
    let locpos = vec3(pos.xz & vec2(0xFu), pos.y);
    let locind = locpos.x | (((locpos.y << 4u) | locpos.z) << 4u);
    return bool((*chunkptr).chunk_mask[locind >> 5u] & (1u << (locind & 0x1Fu)));
};
const AIR = Material(0.0);
fn block_mat(pos: vec3<u32>) -> Material {
    let cpos = pos.xz >> vec2(4u);
    let chunkptr = &chunk_data[(chunk_index_table.data[cpos.x][cpos.y])];
    let locpos = vec3(pos.xz & vec2(0xFu), pos.y);
    let locind = locpos.x | (((locpos.y << 4u) | locpos.z) << 4u);
    let matind = (*chunkptr).chunk_mask[locind >> 5u] & (1u << (locind & 0x1Fu));
    if matind == 0u {return AIR;} else {return (*chunkptr).mats[matind - 1u];}
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vert_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch in_vert_index {
        case 0u: {out.clip_position = vec4(1.0, 1.0, 0.0, 1.0);}
        case 1u: {out.clip_position = vec4(-1.0, 1.0, 0.0, 1.0);}
        case 2u: {out.clip_position = vec4(-1.0, -1.0, 0.0, 1.0);}
        case 3u: {out.clip_position = vec4(1.0, 1.0, 0.0, 1.0);}
        case 4u: {out.clip_position = vec4(-1.0, -1.0, 0.0, 1.0);}
        case 5u: {out.clip_position = vec4(1.0, -1.0, 0.0, 1.0);}
        case default {out.clip_position = vec4(vec3(0.0), 1.0);}
    }
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    return in.clip_position / vec4(vec2(2880.0, 1800.0), 1.0, 1.0);
}



