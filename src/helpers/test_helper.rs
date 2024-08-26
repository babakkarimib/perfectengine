use crate::types::pixel::Pixel;

pub struct TestHelper {}

impl TestHelper {
    pub fn generate_cube_pixels(iters: usize, size: f32) -> (Vec<Pixel>, usize) {
        let colors = [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
            [1.0, 0.0, 1.0, 1.0],
            [0.0, 1.0, 1.0, 1.0],
        ];
    
        let step = (iters as f32).cbrt().floor() as usize;
        let step_size = size / step as f32;
    
        let mut pixels = Vec::new();
        let mut count = 0;
    
        for i in 0..step {
            for j in 0..step {
                for k in 0..step {
                    let is_face = i == 0 || i == step - 1 || j == 0 || j == step - 1 || k == 0 || k == step - 1;
    
                    if !is_face {
                        continue;
                    }
    
                    let pos_x = i as f32 * step_size - size / 2.0 + step_size / 2.0;
                    let pos_y = j as f32 * step_size - size / 2.0 + step_size / 2.0;
                    let pos_z = k as f32 * step_size - size / 2.0 + step_size / 2.0;
    
                    let color = colors[(i + j + k) % colors.len()];
    
                    pixels.push(Pixel {
                        x: pos_x,
                        y: pos_y,
                        z: pos_z,
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: color[3],
                        size_factor: step_size,
                    });
                    count += 1;
                }
            }
        }
    
        (pixels, count)
    }
    
}
