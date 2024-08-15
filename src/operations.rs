pub struct Operations {}

impl Operations {
    pub fn rotate(sx: f32, sy: f32, cx: f32, cy: f32, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let tmp_x = x;
        let tmp_y = cx * y - sx * z;
        let tmp_z = sx * y + cx * z;
    
        let final_x = cy * tmp_x - sy * tmp_z;
        let final_y = tmp_y;
        let final_z = sy * tmp_x + cy * tmp_z;
    
        (final_x, final_y, final_z)
    }
    
    pub fn apply_lighting(
        x: f32,
        y: f32,
        z: f32,
        light_x: f32,
        light_y: f32,
        light_z: f32,
        intensity: f32,
        r: u8,
        g: u8,
        b: u8,
    ) -> (u8, u8, u8) {
        let light_vector = (light_x - x, light_y - y, light_z - z);
    
        let distance = (light_vector.0.powi(2) + light_vector.1.powi(2) + light_vector.2.powi(2)).sqrt();
    
        let adjusted_r = (r as f32 * intensity / distance).min(255.0) as u8;
        let adjusted_g = (g as f32 * intensity / distance).min(255.0) as u8;
        let adjusted_b = (b as f32 * intensity / distance).min(255.0) as u8;
    
        (adjusted_r, adjusted_g, adjusted_b)
    }
    
    pub fn project(
        scale: f32,
        distance: f32,
        canvas_width: f32,
        canvas_height: f32,
        x: f32,
        y: f32,
        z: f32,
    ) -> (u32, u32) {
        let factor = scale / (distance + z);
        (
            (x * factor + canvas_width / 2.0) as u32,
            (-y * factor + canvas_height / 2.0) as u32,
        )
    }
    
    pub fn draw_pixel(
        data: &mut [u8],
        depth_buffer: &mut [f32],
        canvas_width: u32,
        canvas_height: u32,
        x: u32,
        y: u32,
        color: [u8; 4],
        block_size: u32,
        z: f32,
    ) {
        for dx in 0..block_size {
            for dy in 0..block_size {
                let px = x + dx;
                let py = y + dy;
                if px < canvas_width && py < canvas_height {
                    let index = ((py * canvas_width + px) * 4) as usize;
                    let depth_index = (py * canvas_width + px) as usize;
    
                    if z < depth_buffer[depth_index] {
                        depth_buffer[depth_index] = z;
                        data[index] = color[3];
                        data[index + 1] = color[0];
                        data[index + 2] = color[1];
                        data[index + 3] = color[2];
                    }
                }
            }
        }
    }
}
