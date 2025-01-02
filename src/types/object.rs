use super::pixel::Pixel;

pub struct Object {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub angle_x: f32,
    pub angle_y: f32,
    pub angle_z: f32,
    pub pixels: Vec<Pixel>
}
