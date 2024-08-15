use bytemuck::NoUninit;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Uniforms {
    pub sx: f32,
    pub sy: f32,
    pub cx: f32,
    pub cy: f32,
    pub scale: f32,
    pub distance: f32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub light_x: f32,
    pub light_y: f32,
    pub light_z: f32,
    pub intensity: f32,
}

unsafe impl NoUninit for Uniforms {}
