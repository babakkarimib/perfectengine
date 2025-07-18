struct Pixel {
    id: u32,
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
    angle_z: f32,
    c_angle_x: f32,
    c_angle_y: f32,
    c_angle_z: f32,
    l_angle_x: f32,
    l_angle_y: f32,
    l_angle_z: f32,
    scale: f32,
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
    z_offset: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read_write> pixels: array<Pixel>;
@group(0) @binding(2) var<storage, read_write> img: array<u32>;
@group(0) @binding(3) var<storage, read_write> depth_buffer: array<f32>;
@group(0) @binding(4) var<storage, read_write> depth_map_buffer: array<u32>;
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
    color: vec4<f32>
) -> vec3<f32> {
    if (color[3] == -1.0) {
        return vec3<f32>(color[0], color[1], color[2]) * 0.3;
    }
    let distance = distance(light, vec3<f32>(position.x, position.y, position.z));
    let intensity = uniforms.intensity / distance;
    return clamp(vec3<f32>(color[0], color[1], color[2]) * intensity, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn project(v: vec3<f32>, scale_factor: f32) -> vec2<i32> {
    return vec2<i32>(
        i32(v.x * scale_factor + uniforms.canvas_width / 2.0),
        i32(-v.y * scale_factor + uniforms.canvas_height / 2.0)
    );
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let pixel = pixels[index];

    var trasnformed_pixel = rotate(
        vec3<f32>(pixel.x, pixel.y, pixel.z),
        vec3<f32>(uniforms.angle_x, uniforms.angle_y, uniforms.angle_z));
    trasnformed_pixel += vec3<f32>(uniforms.ref_x, uniforms.ref_y, uniforms.ref_z);

    var positioned_pixel = rotate(
        vec3<f32>(trasnformed_pixel.x, trasnformed_pixel.y, trasnformed_pixel.z - uniforms.camera_z),
        vec3<f32>(-uniforms.c_angle_x, -uniforms.c_angle_y, -uniforms.c_angle_z));
    positioned_pixel += vec3<f32>(uniforms.camera_x, uniforms.camera_y, uniforms.camera_z);

    if (uniforms.camera_z - positioned_pixel.z < uniforms.z_offset) {
        return;
    }

    let scale_factor = uniforms.scale / (uniforms.camera_z - positioned_pixel.z);
    let projected = project(positioned_pixel, scale_factor);

    let lit_color = apply_lighting(
        trasnformed_pixel,
        vec3<f32>(uniforms.light_x, uniforms.light_y, uniforms.light_z),
        vec4<f32>(pixel.r, pixel.g, pixel.b, pixel.a));
    let color = vec4<f32>(lit_color, 1.0);

    let canvas_width = i32(uniforms.canvas_width);
    let canvas_height = i32(uniforms.canvas_height);
    let block_size = i32(ceil(scale_factor * pixel.size_factor));
    for (var dx: i32 = 0; dx < block_size; dx++) {
        for (var dy: i32 = 0; dy < block_size; dy++) {
            let px_offset = projected.x + dx;
            let py_offset = projected.y + dy;

            if (px_offset < 0 || px_offset >= canvas_width || py_offset < 0 || py_offset >= canvas_height) {
                continue;
            }

            let depth_index = py_offset * canvas_width + px_offset;
            while (true) {
                if (atomicCompareExchangeWeak(&lock[depth_index], 0u, 1u).exchanged) {
                    if (positioned_pixel.z > depth_buffer[depth_index] || depth_map_buffer[depth_index] == 0u) {
                        depth_buffer[depth_index] = positioned_pixel.z;
                        depth_map_buffer[depth_index] = pixel.id;
                        
                        img[depth_index] = u32(color[0] * 255.0) << 24 |
                                           u32(color[1] * 255.0) << 16 |
                                           u32(color[2] * 255.0) << 8  |
                                           u32(color[3] * 255.0);
                    }
                    atomicStore(&lock[depth_index], 0u);
                    break;
                }
                if (positioned_pixel.z <= depth_buffer[depth_index] && depth_map_buffer[depth_index] > 0u) {
                    break;
                }
            }
        }
    }
}
