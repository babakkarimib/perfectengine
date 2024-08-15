use crate::{light::Light, operations::Operations, pixel::Pixel, renderer::Renderer, view_state::ViewState};

pub struct CpuRenderer<'a> {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture: sdl2::render::Texture<'a>,
    pixels: Vec<Pixel>,
    canvas_width: u32,
    canvas_height: u32
}

impl CpuRenderer<'_> {
    pub fn new(
        canvas: sdl2::render::Canvas<sdl2::video::Window>,
        texture: sdl2::render::Texture,
    ) -> CpuRenderer {
        let (canvas_width, canvas_height) = canvas.output_size().unwrap();

        CpuRenderer {
            canvas,
            texture,
            pixels: Vec::new(),
            canvas_width,
            canvas_height
        }
    }
}

impl Renderer<'_> for CpuRenderer<'_> {
    fn render(&mut self, view_state: &ViewState, light: &Light) {
        let ViewState {
            angle_x,
            angle_y,
            scale,
            distance,
        } = *view_state;

        let mut pixel_data: Vec<u8> = vec![0; (self.canvas_width * self.canvas_height * 4) as usize];
        let mut depth_buffer = vec![f32::INFINITY; (self.canvas_width * self.canvas_height) as usize];

        for pixel in &self.pixels {
            let (sx, sy, cx, cy) = (angle_x.sin(), angle_y.sin(), angle_x.cos(), angle_y.cos());
            let rotated = Operations::rotate(sx, sy, cx, cy, pixel.x, pixel.y, pixel.z);
            let color = Operations::apply_lighting(rotated.0, rotated.1, rotated.2, light.x, light.y, light.z, light.intensity, pixel.r, pixel.g, pixel.b);
            let color = [color.0, color.1, color.2, pixel.a];

            let projected = Operations::project(scale, distance, self.canvas_width as f32, self.canvas_height as f32, rotated.0, rotated.1, rotated.2);
            let block_size = (scale / (distance + rotated.2) * pixel.size_factor).ceil() as u32;

            Operations::draw_pixel(&mut pixel_data, &mut depth_buffer, self.canvas_width, self.canvas_height, projected.0, projected.1, color, block_size, rotated.2);
        }

        self.texture.update(None, &pixel_data, (self.canvas_width * 4) as usize).unwrap();
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }

    fn load_pixels(&mut self, new_pixels: Vec<Pixel>) {
        self.pixels.extend(new_pixels);
    }
}
