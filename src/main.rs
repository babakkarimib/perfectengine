mod types;
mod graphics;
mod events;
mod helpers;

use std::time::{Duration, Instant};
use async_std::print;
use async_std::task;
use helpers::model_helper;
// use helpers::test_helper;
use std::env;

use types::{view_state::ViewState, light::Light, event_callback::EventCallback, renderer::Renderer};
use graphics::{gpu::gpu_renderer::GpuRenderer, cpu::cpu_renderer::CpuRenderer};
use events::event_handler::EventHandler;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const FPS: u32 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS as u64);

#[async_std::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let gpu_enabled = if let Some(arg) = args.get(1) {
        match arg.as_str() {
            "gpu" => true,
            "cpu" => false,
            _ => panic!("Invalid argument: {}. Please use 'gpu' or 'cpu'.", arg),
        }
    } else {
        true
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Perfect Engine", WIDTH, HEIGHT)
        .position_centered()
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    let texture_creator;
    let mut renderer: Box<dyn Renderer<'static>> = if gpu_enabled {
        Box::new(GpuRenderer::new(&window).await)
    } else {
        let canvas = window.into_canvas().present_vsync().build().unwrap();
        texture_creator = canvas.texture_creator();
        Box::new(CpuRenderer::new(canvas, &texture_creator))
    };

    let mut total_pixel_count = 0;
    // let (pixels, pixel_count) = test_helper::generate_cube_pixels(400000, 4.0);
    // renderer.load_pixels(pixels);
    // total_pixel_count += pixel_count;
    let (pixels, pixel_count) = model_helper::load_msh_file_with_texture().await;
    renderer.load_pixels(pixels);
    total_pixel_count += pixel_count;

    let event_pump = sdl_context.event_pump().unwrap();
    let mut event_handler = EventHandler::new(event_pump);

    let mut view_state = ViewState {
        angle_x: 0.0,
        angle_y: 0.0,
        scale: 300.0,
        distance: 250.0
    };
    let mut light = Light {
        x: 20.0,
        y: 0.0,
        z: -110.0,
        intensity: 40.0,
    };

    println!("\nGPU ENABLED:  {}\t\tFPS LIMIT: {:5}\tPIXEL COUNT: {:10}", gpu_enabled, FPS, total_pixel_count);
    'running: loop {
        let process_start = Instant::now();

        let event_callback = event_handler.handle_events(&mut view_state, &mut light);
        match event_callback {
            EventCallback::Quit => break 'running,
            EventCallback::Resized(width, height) => renderer.resize(width, height),
            EventCallback::Next => renderer.render(&view_state, &light)
        }

        let process_duration = process_start.elapsed();
        print!("\rFRAME TIME: {:4}ms\t\tFRAME RATE: {:4}", process_duration.as_millis(), 1000 / process_duration.as_millis()).await;
        if process_duration < FRAME_DURATION {
            task::sleep(FRAME_DURATION - process_duration).await;
        }
    }
}
