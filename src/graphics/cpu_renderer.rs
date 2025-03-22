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
        let size = (self.canvas_width * self.canvas_height) as usize;
        let mut pixel_map: Vec<i32> = vec![-1; size];
        let mut pixel_transformations: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); size];
        let mut depth_buffer = vec![-f32::INFINITY; size];

        for (i, pixel) in self.pixels.iter().enumerate() {
            let mut trasnformed_pixel = Operations::rotate(
                (pixel.x, pixel.y, pixel.z),
                (view_state.angle_x, view_state.angle_y, view_state.angle_z)
            );
            trasnformed_pixel.0 += view_state.ref_x;
            trasnformed_pixel.1 += view_state.ref_y;
            trasnformed_pixel.2 += view_state.ref_z;

            pixel_transformations.push(trasnformed_pixel);

            let mut positioned_pixel = Operations::rotate(
                (
                    trasnformed_pixel.0, 
                    trasnformed_pixel.1, 
                    trasnformed_pixel.2 - view_state.camera_z
                ),
                (-view_state.c_angle_x, -view_state.c_angle_y, -view_state.c_angle_z)
            );

            positioned_pixel.0 += view_state.camera_x;
            positioned_pixel.1 += view_state.camera_y;
            positioned_pixel.2 += view_state.camera_z;

            if view_state.camera_z - positioned_pixel.2 < view_state.z_offset { continue; }

            let scale_factor = view_state.scale / (view_state.camera_z - view_state.ref_z);

            let projected = Operations::project(
                positioned_pixel,
                scale_factor,
                self.canvas_width as f32, 
                self.canvas_height as f32
            );

            let block_size = (scale_factor * pixel.size_factor).ceil() as i32;

            Operations::draw_pixel(&mut pixel_map, &mut depth_buffer, self.canvas_width as i32, self.canvas_height as i32, projected.0, projected.1, block_size, positioned_pixel.2, i as u32);
        }

        let pixel_data: Vec<u8> = pixel_map.iter().flat_map(|&index| {
            if index == -1 { return vec![0, 0, 0, 0]}
            let index = index as usize;
            let pixel = self.pixels[index];

            let lit_color = Operations::apply_lighting(
                pixel_transformations[index],
                (pixel.r, pixel.g, pixel.b), 
                (light.x, light.y, light.z), 
                light.intensity
            );

            vec![
                (pixel.a * 255.0) as u8,
                (lit_color.0 * 255.0) as u8,
                (lit_color.1 * 255.0) as u8,
                (lit_color.2 * 255.0) as u8,
            ]
        }).collect();        
        
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
