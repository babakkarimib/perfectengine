struct Pixel {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    size_factor: f32,
};

struct Uniforms {
    sx: f32,
    sy: f32,
    cx: f32,
    cy: f32,
    scale: f32,
    distance: f32,
    canvas_width: f32,
    canvas_height: f32,
    light_x: f32,
    light_y: f32,
    light_z: f32,
    intensity: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read_write> pixels: array<Pixel>;
@group(0) @binding(2) var img: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(3) var<storage, read_write> depth_buffer: array<f32>;
@group(0) @binding(4) var<storage, read_write> depth_check_buffer: array<u32>;
@group(0) @binding(5) var<storage, read_write> lock: array<atomic<u32>>;

fn rotate(v: vec3<f32>) -> vec3<f32> {
    let tmp_x = v.x;
    let tmp_y = uniforms.cx * v.y - uniforms.sx * v.z;
    let tmp_z = uniforms.sx * v.y + uniforms.cx * v.z;

    let final_x = uniforms.cy * tmp_x - uniforms.sy * tmp_z;
    let final_y = tmp_y;
    let final_z = uniforms.sy * tmp_x + uniforms.cy * tmp_z;

    return vec3<f32>(final_x, final_y, final_z);
}

fn apply_lighting(
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
) -> vec3<f32> {
    let light_vector = vec3<f32>(uniforms.light_x - x, uniforms.light_y - y, uniforms.light_z - z);
    let distance = length(light_vector);

    let adjusted_r = clamp(r * uniforms.intensity / distance, 0.0, 1.0);
    let adjusted_g = clamp(g * uniforms.intensity / distance, 0.0, 1.0);
    let adjusted_b = clamp(b * uniforms.intensity / distance, 0.0, 1.0);

    return vec3<f32>(adjusted_b, adjusted_g, adjusted_r);
}

fn project(v: vec3<f32>) -> vec2<f32> {
    let factor = uniforms.scale / (uniforms.distance + v.z);
    return vec2<f32>(v.x * factor + uniforms.canvas_width / 2.0, -v.y * factor + uniforms.canvas_height / 2.0);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let pixel = pixels[index];
    let rotated = rotate(vec3<f32>(pixel.x, pixel.y, pixel.z));
    let lit_color = apply_lighting(rotated.x, rotated.y, rotated.z, pixel.r, pixel.g, pixel.b);
    let projected = project(rotated);
    let color = vec4<f32>(lit_color, pixel.a);

    let px = i32(projected.x);
    let py = i32(projected.y);
    let block_size = i32(ceil(uniforms.scale / (uniforms.distance + rotated.z) * pixel.size_factor));
    for (var dx: i32 = 0; dx < block_size; dx++) {
        for (var dy: i32 = 0; dy < block_size; dy++) {
            let px_offset = px + dx;
            let py_offset = py + dy;
            let depth_index = py_offset * i32(uniforms.canvas_width) + px_offset;
            while (true) {
                if (atomicCompareExchangeWeak(&lock[depth_index], 0u, 1u).exchanged) {
                    if (rotated.z < depth_buffer[depth_index] || depth_check_buffer[depth_index] == 0u) {
                        depth_check_buffer[depth_index] = 1u;
                        depth_buffer[depth_index] = rotated.z;
                        textureStore(img, vec2<i32>(px_offset, py_offset), color);
                    }
                    atomicStore(&lock[depth_index], 0u);
                    break;
                }
                if (rotated.z >= depth_buffer[depth_index] && depth_check_buffer[depth_index] == 1u) {
                    break;
                }
            }
        }
    }
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
