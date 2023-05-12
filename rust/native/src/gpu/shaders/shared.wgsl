const RESC_WORLD_SIDE = 7u;
struct Material {
    property_idk: f32,
}
// 24576 = 16 * 16 * 384 / 4 (16x16x384 chunk, 1byte material refs, 4 bytes per u32)
struct Chunk {
    chunk_mrefs: array<u32, 24576>,
}
struct Uniforms {
    chunk_index_table: array<vec4<u32>, 256u>,
}
@group(0) @binding(2)
var<uniform> uniforms: Uniforms;
//DANGEROUS! CAN INDEX OUT OF BOUNDS IF INPUTS ARE NOT <16
fn chunk_index(cx: u32, cz: u32) -> u32 {
    let ind = (cx << 4u) | cz;
    switch ind & 3u {
        case 0u: {return uniforms.chunk_index_table[ind].x;}
        case 1u: {return uniforms.chunk_index_table[ind].y;}
        case 2u: {return uniforms.chunk_index_table[ind].z;}
        case 3u: {return uniforms.chunk_index_table[ind].w;}
        case default: {return 0u;}
    }
}

@group(0) @binding(0)
var<storage, read> chunks: array<Chunk>;

@group(0) @binding(1)
var<storage, read> materials: array<Material>;

// ! ASSUMES VALID COORDINATES ! ! BE CAREFUL !
fn block_mref(pos: vec3<u32>) -> u32 {
    let cpos = pos.xz >> vec2(4u);
    let chunkptr = &chunks[chunk_index(cpos.x, cpos.y)];
    let locpos = vec3(pos.xz & vec2(0xFu), pos.y);
    let locind = locpos.x | (((locpos.y << 4u) | locpos.z) << 4u);
    let l = locind & 1u;
    var out = 0u;
    let dat = (*chunkptr).chunk_mrefs[locind >> 1u];
    out += l * (dat >> 16u);
    out += (1u-l) * (dat & 0xFFFFu);
    return out;
}


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



