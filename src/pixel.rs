use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Pixel {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub size_factor: f32,
}

unsafe impl Zeroable for Pixel {}
unsafe impl Pod for Pixel {}
