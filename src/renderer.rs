use crate::{light::Light, pixel::Pixel, view_state::ViewState};

pub trait Renderer<'a> {
    fn render(&mut self, view_state: &ViewState, light: &Light);
    fn load_pixels(&mut self, new_pixels: Vec<Pixel>);    
    fn resize(&mut self, width: u32, height: u32);
}
