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
[[group(0), binding(0)]]
var<storage, read> chunk_index_table: ChunkIndexTable;
[[group(0), binding(1)]]
var<storage, read> chunk_data: array<Chunk>;

fn block_mref(pos: vec3<u32>) -> u32 {
    let cpos = pos.xz >> vec2(4u);
    let chunk = *&chunk_data[(chunk_index_table.data[cpos.x][cpos.y])];
    let locpos = vec3(pos.xz & vec2(0xFu), pos.y);
    let locind = locpos.x | (((locpos.y << 4u) | locpos.z) << 4u);
    bool(chunk.chunk_mask[locind >> 5u] & (1u << (locind & 0x1Fu)))
}
const AIR = Material(0.0);
fn block_mat(pos: vec3<u32>) -> Material {
    let cpos = pos.xz >> vec2(4u);
    let chunk = *&chunk_data[(chunk_index_table.data[cpos.x][cpos.y])];
    let locpos = vec3(pos.xz & vec2(0xFu), pos.y);
    let locind = locpos.x | (((locpos.y << 4u) | locpos.z) << 4u);
    let matind = chunk.chunk_mask[locind >> 5u] & (1u << (locind & 0x1Fu));
    if matind == 0u {AIR} else {chunk.mats[matind - 1u]}
}

