const RESC_WORLD_RADIUS = 3u;
struct Material {
    property_idk: f32,
}
// 24576 = 16 * 16 * 384 / 2 (16x16x384 chunk, 2byte material refs, 4 bytes per u32)
struct Chunk {
    chunk_mrefs: array<u32, 49152>,
}
struct Uniforms {
    //We pack 4 array entries at every index due to offset restraints
    chunk_index_table: array<vec4<u32>, 256u>,
}
@group(0) @binding(2)
var<uniform> uniforms: Uniforms;
//DANGEROUS! CAN INDEX OUT OF BOUNDS IF INPUTS ARE NOT <16
fn chunk_index(pos: vec2<i32>) -> u32 {
    var pos2 = pos;
    pos2 += vec2<i32>(i32(RESC_WORLD_RADIUS));
    let ind = ((u32(pos2.y)) << 6u) | u32(pos2.x);
    switch(ind & 3u) {
        case 0u: {return uniforms.chunk_index_table[ind >> 2u].x;}
        case 1u: {return uniforms.chunk_index_table[ind >> 2u].y;}
        case 2u: {return uniforms.chunk_index_table[ind >> 2u].z;}
        case 3u: {return uniforms.chunk_index_table[ind >> 2u].w;}
        default: {return 0u;}
    }
}

@group(0) @binding(0)
var<storage, read> chunks: array<Chunk>;

@group(0) @binding(1)
var<storage, read> materials: array<Material>;

// ! ASSUMES VALID COORDINATES ! ! BE CAREFUL !
fn block_mref(pos: vec3<i32>) -> u32 {
    let cpos = pos.xz / 16;
    let chunkptr = &chunks[chunk_index(cpos)];
    let locpos = vec3(vec2<u32>(pos.xz) & vec2(0xFu), u32(pos.y));
    let locind = locpos.x | (((locpos.y << 4u) | locpos.z) << 4u);
    let l = locind & 1u;
    var out = 0u;
    let dat = (*chunkptr).chunk_mrefs[locind >> 1u];
    out += l * (dat >> 16u);
    out += (1u - l) * (dat & 0xFFFFu);
    return out;
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vert_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch in_vert_index {
    //Awesome hardcoded quad
        case 0u: {out.clip_position = vec4(1.0, 1.0, 0.0, 1.0);}
        case 1u: {out.clip_position = vec4(-1.0, 1.0, 0.0, 1.0);}
        case 2u: {out.clip_position = vec4(-1.0, -1.0, 0.0, 1.0);}
        case 3u: {out.clip_position = vec4(1.0, 1.0, 0.0, 1.0);}
        case 4u: {out.clip_position = vec4(-1.0, -1.0, 0.0, 1.0);}
        case 5u: {out.clip_position = vec4(1.0, -1.0, 0.0, 1.0);}
        default: {out.clip_position = vec4(vec3(0.0), 1.0);}
    }
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    return in.clip_position / vec4(vec2(1920.0, 1080.0), 1.0, 1.0);
}



