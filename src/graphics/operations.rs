pub struct Operations {}

impl Operations {
    pub fn rotate(v: (f32, f32, f32), angle: (f32, f32, f32)) -> (f32, f32, f32) {
        let (x, y, z) = v;
        let (angle_x, angle_y, angle_z) = angle;
    
        let cos_x = angle_x.cos();
        let sin_x = angle_x.sin();
        let cos_y = angle_y.cos();
        let sin_y = angle_y.sin();
        let cos_z = angle_z.cos();
        let sin_z = angle_z.sin();
    
        let tmp_y = cos_x * y - sin_x * z;
        let tmp_z = sin_x * y + cos_x * z;
        let rotated_x = (x, tmp_y, tmp_z);
    
        let tmp_x = cos_y * rotated_x.0 + sin_y * rotated_x.2;
        let final_z = -sin_y * rotated_x.0 + cos_y * rotated_x.2;
        let rotated_y = (tmp_x, rotated_x.1, final_z);
    
        let final_x = cos_z * rotated_y.0 - sin_z * rotated_y.1;
        let final_y = sin_z * rotated_y.0 + cos_z * rotated_y.1;
    
        (final_x, final_y, rotated_y.2)
    }
    
    pub fn project(v: (f32, f32, f32), scale_factor: f32, canvas_width: f32, canvas_height: f32) -> (i32, i32) {
        let (x, y, _) = v;
    
        let projected_x = x * scale_factor + canvas_width / 2.0;
        let projected_y = -y * scale_factor + canvas_height / 2.0;
    
        (projected_x as i32, projected_y as i32)
    }

    pub fn draw_pixel(
        data: &mut [i32],
        depth_buffer: &mut [f32],
        canvas_width: i32,
        canvas_height: i32,
        x: i32,
        y: i32,
        block_size: i32,
        z: f32,
        index: u32,
    ) {
        for dx in 0..block_size {
            for dy in 0..block_size {
                let px_offset = x + dx;
                let py_offset = y + dy;

                if px_offset < 0 || px_offset >= canvas_width || py_offset < 0 || py_offset >= canvas_height {
                    continue;
                }

                let depth_index = (py_offset * canvas_width + px_offset) as usize;
                if z > depth_buffer[depth_index] {
                    depth_buffer[depth_index] = z;
                    data[depth_index] = index as i32;
                }
            }
        }
    }

    pub fn apply_lighting(position: (f32, f32, f32), color: (f32, f32, f32), light_position: (f32, f32, f32), intensity: f32) -> (f32, f32, f32) {
        let (x, y, z) = position;
        let (light_x, light_y, light_z) = light_position;
    
        let distance = ((light_x - x).powi(2) + (light_y - y).powi(2) + (light_z - z).powi(2)).sqrt();
        let attenuation = intensity / distance;
    
        let adjusted_r = (color.0 * attenuation).clamp(0.0, 1.0);
        let adjusted_g = (color.1 * attenuation).clamp(0.0, 1.0);
        let adjusted_b = (color.2 * attenuation).clamp(0.0, 1.0);
    
        (adjusted_b, adjusted_g, adjusted_r)
    }
}
