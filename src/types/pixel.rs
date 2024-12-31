use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Pixel {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub size_factor: f32,
}

unsafe impl Zeroable for Pixel {}
unsafe impl Pod for Pixel {}
