use crate::{light::Light, pixel::Pixel, view_state::ViewState};

pub trait Renderer<'a> {
    fn render(&mut self, view_state: &ViewState, light: &Light);
    fn load_pixels(&mut self, new_pixels: Vec<Pixel>);    
}
