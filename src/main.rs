mod cpu_renderer;
mod test_helper;
mod event_handler;
mod gpu_renderer;
mod operations;
mod pixel;
mod uniforms;
mod view_state;
mod light;
mod renderer;
mod event_callback;

use async_std::print;
use async_std::task;
use event_callback::EventCallback;
use std::time::{Duration, Instant};

use cpu_renderer::CpuRenderer;
use event_handler::EventHandler;
use gpu_renderer::GpuRenderer;
use light::Light;
use renderer::Renderer;
use view_state::ViewState;
use crate::test_helper::TestHelper;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const GPU_ENABLED: bool = false;

#[async_std::main]
async fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Perfect Engine", WIDTH, HEIGHT)
        .position_centered()
        .resizable()
        .vulkan()
        .build()
        .unwrap();

    let texture_creator;
    let mut renderer: Box<dyn Renderer<'_>> = if GPU_ENABLED {
        Box::new(GpuRenderer::new(&window).await)
    } else {
        let canvas = window.into_canvas().build().unwrap();
        texture_creator = canvas.texture_creator();
        Box::new(CpuRenderer::new(canvas, &texture_creator))
    };

    let pixels = TestHelper::generate_cube_pixels(2000000, 6.0);
    renderer.load_pixels(pixels);

    let event_pump = sdl_context.event_pump().unwrap();
    let mut event_handler = EventHandler::new(event_pump);

    let mut view_state = ViewState {
        angle_x: 0.0,
        angle_y: 0.0,
        scale: 300.0,
        distance: 10.0
    };
    let mut light = Light {
        x: 5.0,
        y: 4.0,
        z: -4.0,
        intensity: 3.5,
    };

    println!("\nGPU ENABLED: {}", GPU_ENABLED);
    let frame_duration = Duration::from_millis(16);
    'running: loop {
        let process_start = Instant::now();

        let event_callback = event_handler.handle_events(&mut view_state, &mut light);
        match event_callback {
            EventCallback::QUIT => break 'running,
            EventCallback::RESIZE(width, height) => renderer.resize(width, height),
            EventCallback::NEXT => renderer.render(&view_state, &light)
        }

        let process_duration = process_start.elapsed();
        print!("\rFRAME DURATION: {:2}ms ", process_duration.as_millis()).await;
        if process_duration < frame_duration {
            task::sleep(frame_duration - process_duration).await;
        }
    }
}
