use sdl2::{pixels::PixelFormatEnum, render::TextureCreator, video::WindowContext};
use crate::types::{light::Light, pixel::Pixel, renderer::Renderer, view_state::ViewState};
use super::operations::Operations;

pub struct CpuRenderer<'a> {
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
    texture: sdl2::render::Texture<'a>,
    pixels: Vec<Pixel>,
    canvas_width: u32,
    canvas_height: u32,
}

impl CpuRenderer<'_> {
    pub fn new<'a>(
        canvas: sdl2::render::Canvas<sdl2::video::Window>,
        texture_creator: &'a TextureCreator<WindowContext>,
    ) -> CpuRenderer<'a> {
        let (canvas_width, canvas_height) = canvas.output_size().unwrap();
        let texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, canvas_width, canvas_height)
        .unwrap();
        CpuRenderer {
            canvas,
            texture_creator,
            texture,
            pixels: Vec::new(),
            canvas_width,
            canvas_height,
        }
    }
}

impl Renderer<'_> for CpuRenderer<'_> {
    fn render(&mut self, view_state: &ViewState, light: &Light) {
        let mut pixel_data: Vec<u8> = vec![0; (self.canvas_width * self.canvas_height * 4) as usize];
        let mut depth_buffer = vec![-f32::INFINITY; (self.canvas_width * self.canvas_height) as usize];

        for pixel in &self.pixels {
            let mut rotated_pixel = Operations::rotate(
                (pixel.x, pixel.y, pixel.z),
                (view_state.angle_x, view_state.angle_y, view_state.angle_z)
            );
            rotated_pixel.0 += view_state.ref_x;
            rotated_pixel.1 += view_state.ref_y;
            rotated_pixel.2 += view_state.ref_z;

            let mut rotated_position = Operations::rotate(
                (
                    rotated_pixel.0, 
                    rotated_pixel.1, 
                    rotated_pixel.2 - view_state.camera_z
                ),
                (-view_state.c_angle_x, -view_state.c_angle_y, -view_state.c_angle_z)
            );
            rotated_position.0 += view_state.camera_x;
            rotated_position.1 += view_state.camera_y;
            rotated_position.2 += view_state.camera_z;

            let aspect_ratio = std::cmp::max(self.canvas_width, self.canvas_height) as f32 / std::cmp::min(self.canvas_width, self.canvas_height) as f32;
            let distance_z = view_state.camera_z - rotated_position.2;
            let scale_factor = view_state.scale / distance_z * aspect_ratio * 2.0;
            if scale_factor > distance_z { continue; }

            let lit_color = Operations::apply_lighting(
                rotated_pixel,
                (pixel.r, pixel.g, pixel.b), 
                (light.x, light.y, light.z), 
                light.intensity
            );

            let projected = Operations::project(
                rotated_position,
                scale_factor,
                self.canvas_width as f32, 
                self.canvas_height as f32
            );

            let color = [lit_color.0, lit_color.1, lit_color.2, pixel.a];
            let block_size = (scale_factor * pixel.size_factor).ceil() as i32;

            Operations::draw_pixel(&mut pixel_data, &mut depth_buffer, self.canvas_width as i32, self.canvas_height as i32, projected.0, projected.1, color, block_size, rotated_position.2);
        }

        self.texture.update(None, &pixel_data, (self.canvas_width * 4) as usize).unwrap();
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None).unwrap();
        self.canvas.present();
    }

    fn load_pixels(&mut self, new_pixels: Vec<Pixel>) {
        self.pixels.extend(new_pixels);
    }
    
    fn resize(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
        self.texture = self.texture_creator
        .create_texture_streaming(PixelFormatEnum::RGBA8888, width, height)
        .unwrap();
    }
}
