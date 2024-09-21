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
    angle_x: f32,
    angle_y: f32,
    c_angle_x: f32,
    c_angle_y: f32,
    scale: f32,
    perspective_factor: f32,
    focal_distance: f32,
    canvas_width: f32,
    canvas_height: f32,
    light_x: f32,
    light_y: f32,
    light_z: f32,
    intensity: f32,
    camera_x: f32,
    camera_y: f32,
    camera_z: f32,
    ref_x: f32,
    ref_y: f32,
    ref_z: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read_write> pixels: array<Pixel>;
@group(0) @binding(2) var img: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(3) var<storage, read_write> depth_buffer: array<f32>;
@group(0) @binding(4) var<storage, read_write> depth_check_buffer: array<u32>;
@group(0) @binding(5) var<storage, read_write> lock: array<atomic<u32>>;

fn rotate(v: vec3<f32>, angle: vec3<f32>) -> vec3<f32> {
    let cos_x = cos(angle.x);
    let sin_x = sin(angle.x);
    let cos_y = cos(angle.y);
    let sin_y = sin(angle.y);
    let cos_z = cos(angle.z);
    let sin_z = sin(angle.z);

    let tmp_y = cos_x * v.y - sin_x * v.z;
    let tmp_z = sin_x * v.y + cos_x * v.z;
    let rotated_x = vec3<f32>(v.x, tmp_y, tmp_z);

    let tmp_x = cos_y * rotated_x.x + sin_y * rotated_x.z;
    let final_z = -sin_y * rotated_x.x + cos_y * rotated_x.z;
    let rotated_y = vec3<f32>(tmp_x, rotated_x.y, final_z);

    let final_x = cos_z * rotated_y.x - sin_z * rotated_y.y;
    let final_y = sin_z * rotated_y.x + cos_z * rotated_y.y;

    return vec3<f32>(final_x, final_y, rotated_y.z);
}

fn apply_lighting(
    position: vec3<f32>,
    light: vec3<f32>,
    color: vec3<f32>
) -> vec3<f32> {
    let distance = distance(light, vec3<f32>(position.x, position.y, position.z));
    let intensity = uniforms.intensity / distance;
    return clamp(color * intensity, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn project(v: vec3<f32>, scale_factor: f32) -> vec2<f32> {
    return vec2<f32>(
        (v.x + uniforms.camera_x) * scale_factor + uniforms.canvas_width / 2.0,
        -(v.y + uniforms.camera_y) * scale_factor + uniforms.canvas_height / 2.0
    );
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let pixel = pixels[index];

    var rotated_pixel = rotate(vec3<f32>(pixel.x, pixel.y, pixel.z), vec3<f32>(uniforms.angle_x, uniforms.angle_y, 0.0));
    rotated_pixel -= vec3<f32>(uniforms.ref_x, uniforms.ref_y, uniforms.ref_z);

    let rotated_light = rotate(
        vec3<f32>(uniforms.light_x - (uniforms.camera_x / uniforms.camera_z), uniforms.light_y - (uniforms.camera_y / uniforms.camera_z), uniforms.light_z), 
        vec3<f32>(uniforms.c_angle_x, uniforms.c_angle_y, 0.0));
    let lit_color = apply_lighting(rotated_pixel, rotated_light, vec3<f32>(pixel.r, pixel.g, pixel.b));
    
    var rotated_position = rotate(
        vec3<f32>(rotated_pixel.x, rotated_pixel.y, rotated_pixel.z + uniforms.focal_distance), 
        vec3<f32>(uniforms.c_angle_x, uniforms.c_angle_y, 0.0));
    rotated_position.z -= uniforms.focal_distance;

    let scale_factor = uniforms.scale / (uniforms.camera_z + rotated_position.z * uniforms.perspective_factor);

    let projected = project(rotated_position, scale_factor);

    let color = vec4<f32>(lit_color, pixel.a);
    let px = i32(projected.x);
    let py = i32(projected.y);
    let block_size = i32(ceil(scale_factor * pixel.size_factor));

    for (var dx: i32 = 0; dx < block_size; dx++) {
        for (var dy: i32 = 0; dy < block_size; dy++) {
            let px_offset = px + dx;
            let py_offset = py + dy;
            let depth_index = py_offset * i32(uniforms.canvas_width) + px_offset;
            while (true) {
                if (atomicCompareExchangeWeak(&lock[depth_index], 0u, 1u).exchanged) {
                    if (rotated_position.z < depth_buffer[depth_index] || depth_check_buffer[depth_index] == 0u) {
                        depth_check_buffer[depth_index] = 1u;
                        depth_buffer[depth_index] = rotated_position.z;
                        textureStore(img, vec2<i32>(px_offset, py_offset), color);
                    }
                    atomicStore(&lock[depth_index], 0u);
                    break;
                }
                if (rotated_position.z >= depth_buffer[depth_index] && depth_check_buffer[depth_index] == 1u) {
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
