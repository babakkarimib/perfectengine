use bytemuck::NoUninit;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Uniforms {
    pub angle_x: f32,
    pub angle_y: f32,
    pub angle_z: f32,
    pub c_angle_x: f32,
    pub c_angle_y: f32,
    pub c_angle_z: f32,
    pub l_angle_x: f32,
    pub l_angle_y: f32,
    pub l_angle_z: f32,
    pub scale: f32,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub light_x: f32,
    pub light_y: f32,
    pub light_z: f32,
    pub intensity: f32,
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_z: f32,
    pub ref_x: f32,
    pub ref_y: f32,
    pub ref_z: f32,
    pub z_offset: f32,
}

unsafe impl NoUninit for Uniforms {}
