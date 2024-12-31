#![allow(dead_code)]
use std::env;
use async_std::fs;
use image::GenericImageView;
use crate::types::{pixel::Pixel, view_state::ViewState};
use super::operations::Operations;

pub async fn load_msh_file_with_texture(id: u32) -> (Vec<Pixel>, usize) {
    let mut pixels: Vec<Pixel> = vec![];
    let current_dir = env::current_dir().expect("Failed to get current directory");
    
    let mesh_file_path = current_dir.join("src").join("helpers").join("model_helper").join("mesh_output.msh");
    let msh_bytes = fs::read(mesh_file_path).await.unwrap();
    let parser_result = mshio::parse_msh_bytes(msh_bytes.as_slice());

    let texture_file_path = current_dir.join("src").join("helpers").join("model_helper").join("texture.png");
    let img = fs::read(texture_file_path).await.expect("Failed to read image");
    let img = image::load_from_memory(&img).expect("Failed to decode image").to_rgba8();

    let width = 800.0;
    let height = 800.0;

    let mut count = 0;

    if let Some(nodes) = parser_result.unwrap().data.nodes {
        for node_block in nodes.node_blocks {
            for node in node_block.nodes {
                let x = node.x as f32;
                let y = node.y as f32;
                let z = node.z as f32;

                let angle_x: f32 = -89.75;
                let angle_y: f32 = 0.0;
                let (rx, ry, rz) = Operations::rotate(
                    (x, y, z), 
                    (angle_x, angle_y, 0.0)
                );
                let scale_factor = 280.0 / (220.0 + rz * 0.0);
                let (tx, ty) = Operations::project(
                    (rx, ry, rz), 
                    scale_factor,
                    width as f32, 
                    height as f32
                );
                let rgba = img.get_pixel(tx as u32, ty as u32);

                pixels.push(Pixel {
                    id,
                    x: rx,
                    y: ry,
                    z: rz,
                    r: rgba[0] as f32 / 255.0,
                    g: rgba[1] as f32 / 255.0,
                    b: rgba[2] as f32 / 255.0,
                    a: rgba[3] as f32 / 255.0,
                    size_factor: 0.5,
                });

                count += 1;
            }
        }
    }

    let view_state = ViewState { angle_x: 0.0, angle_y: 0.0, angle_z: 0.0, scale: 300.0, c_angle_x: 0.0, c_angle_y: 0.0, c_angle_z: 0.0, camera_x: 0.0, camera_y: 0.0, camera_z: 150.0, ref_x: 0.0, ref_y: 0.0, ref_z: 0.0, z_offset: 30.0, };
    load_texture(&mut pixels, count, view_state, width, height, "flower.png", 0, 40).await;

    let view_state = ViewState { angle_x: 0.45, angle_y: 85.0, angle_z: 0.0, scale: 300.0, c_angle_x: 0.0, c_angle_y: 0.0, c_angle_z: 0.0, camera_x: 0.0, camera_y: 0.0, camera_z: 150.0, ref_x: 0.0, ref_y: 0.0, ref_z: 0.0, z_offset: 30.0, };
    load_texture(&mut pixels, count, view_state, width, height, "flower.png",0, 40).await;

    (pixels, count)
}

async fn load_texture(pixels: &mut Vec<Pixel>, count: usize, view_state: ViewState, width: f32, height: f32, path: &str, wd: u32, hd: u32) {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let texture_file_path = current_dir.join("src").join("helpers").join("model_helper").join(path);
    let img = fs::read(texture_file_path).await.expect("Failed to read image");
    let img = image::load_from_memory(&img).expect("Failed to decode image").to_rgba8();
    
    let (f_width, f_height) = img.dimensions();
    let w_disposition = (width as u32 / 2) - (f_width / 2) + wd;
    let h_disposition = (height as u32 / 2) - (f_height / 2) + hd;

    for i in 0..count {
        let pixel = &mut pixels[i];

        let (rx, ry, rz) = Operations::rotate(
            (pixel.x, pixel.y, pixel.z), 
            (view_state.angle_x, view_state.angle_y, 0.0) 
        );
        let scale_factor = view_state.scale / (view_state.camera_z - rz);
        let (tx, ty) = Operations::project(
            (rx, ry, rz), 
            scale_factor,
            width as f32, 
            height as f32
        );

        if rz < 0.0 && img.in_bounds(tx as u32 - w_disposition, ty as u32 - h_disposition) {
            let rgba = img.get_pixel(tx as u32 - w_disposition, ty as u32 - h_disposition);
            if rgba[3] != 0 {
                pixel.r = rgba[0] as f32 / 255.0;
                pixel.g = rgba[1] as f32 / 255.0;
                pixel.b = rgba[2] as f32 / 255.0;
                pixel.a = rgba[3] as f32 / 255.0;
            }
        }
    }
}
