mod types;
mod graphics;
mod events;
mod helpers;

use std::time::{Duration, Instant};
use async_std::print;
use async_std::task;
use regex::Regex;
use std::env;

use types::{view_state::ViewState, light::Light, event_callback::EventCallback, renderer::Renderer};
use graphics::{gpu::gpu_renderer::GpuRenderer, cpu::cpu_renderer::CpuRenderer};
use events::event_handler::EventHandler;

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;

const FPS: u32 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

#[async_std::main]
async fn main() {
    let mut gpu_enabled = true;
    let mut framerate_log = false;
    let mut fullscreen = false;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;

    let width_regex = Regex::new(r"^w=(\d+)$").unwrap();
    let height_regex = Regex::new(r"^h=(\d+)$").unwrap();

    let args: Vec<String> = env::args().collect();
    for i in 1..args.len() {
        if let Some(arg) = args.get(i) {
            match arg.as_str() {
                "gpu" => gpu_enabled = true,
                "cpu" => gpu_enabled = false,
                "framerate" => framerate_log = true,
                "fullscreen" => fullscreen = true,
                _ if width_regex.is_match(arg) => {
                    if let Some(caps) = width_regex.captures(arg) {
                        width = caps.get(1).map(|m| m.as_str().parse().unwrap());
                    }
                }
                _ if height_regex.is_match(arg) => {
                    if let Some(caps) = height_regex.captures(arg) {
                        height = caps.get(1).map(|m| m.as_str().parse().unwrap());
                    }
                }
                _ => panic!("Invalid argument: {}. Please use 'gpu', 'cpu', 'framerate', 'fullscreen', 'w={{}}', or 'h={{}}'.", arg),
            }
        }
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let display_mode = video_subsystem.desktop_display_mode(0).unwrap();

    let width = if let Some(w) = width {w} else { if fullscreen { display_mode.w as u32 } else {DEFAULT_WIDTH} };
    let height = if let Some(h) = height {h} else { if fullscreen { display_mode.h as u32 } else {DEFAULT_HEIGHT} };

    let window = if fullscreen {
        video_subsystem.window("Perfect Engine", width, height)
        .fullscreen()
        .build()
        .unwrap()
    } else {
        video_subsystem.window("Perfect Engine", width, height)
        .position_centered()
        .resizable()
        .build()
        .unwrap()
    };

    let texture_creator;
    let mut renderer: Box<dyn Renderer<'static>> = if gpu_enabled {
        Box::new(GpuRenderer::new(&window).await)
    } else {
        let canvas = window.into_canvas().present_vsync().build().unwrap();
        texture_creator = canvas.texture_creator();
        Box::new(CpuRenderer::new(canvas, &texture_creator))
    };

    let mut pixel_count = 0;
    let (pixels, count) = helpers::test_helper::generate_cube_pixels(200000, 1000.0);
    pixel_count += count;
    renderer.load_pixels(pixels);
    // let (pixels, count) = helpers::model_helper::load_msh_file_with_texture().await;
    // pixel_count += count;
    // renderer.load_pixels(pixels);

    let event_pump = sdl_context.event_pump().unwrap();
    let mut event_handler = EventHandler::new(event_pump);

    let mut view_state = ViewState { 
        angle_x: 0.0,
        angle_y: 0.0,
        angle_z: 0.0,
        c_angle_x: 0.0,
        c_angle_y: 0.0,
        c_angle_z: 0.0,
        camera_x: 0.0,
        camera_y: 0.0,
        camera_z: 600.0,
        ref_x: 0.0,
        ref_y: 0.0,
        ref_z: 0.0,
        scale: 300.0,
    };
    let mut light = Light {
        x: 30.0,
        y: 0.0,
        z: 100.0,
        intensity: 60.0,
    };

    println!("\nFULLSCREEN:   {}\t\tWIDTH: {}\t\tHEIGHT: {}", fullscreen, width, height);
    println!("GPU ENABLED:  {}\t\tFPS LIMIT: {:5}\tPIXEL COUNT: {:10}", gpu_enabled, FPS, pixel_count);
    'running: loop {
        let process_start = Instant::now();

        let event_callback = event_handler.handle_events(&mut view_state, &mut light);
        match event_callback {
            EventCallback::Quit => break 'running,
            EventCallback::Resized(width, height) => renderer.resize(width, height),
            EventCallback::Next => renderer.render(&view_state, &light)
        }

        let process_duration = process_start.elapsed();
        if framerate_log {
            print!("\rFRAME TIME: {:4}ms\t\tFRAME RATE: {:4}", process_duration.as_millis(), 1000 / process_duration.as_millis()).await;
        }
        if process_duration < FRAME_DURATION {
            task::sleep(FRAME_DURATION - process_duration).await;
        }
    }
}
