use sdl2::{event::{Event, WindowEvent}, keyboard::Keycode, mouse::MouseButton, EventPump};
use crate::types::{event_callback::EventCallback, light::Light, view_state::ViewState};

pub struct EventHandler {
    event_pump: EventPump,
    last_x: i32,
    last_y: i32,
    is_dragging: bool,
    is_right_dragging: bool,
}

impl EventHandler {
    pub fn new(event_pump: EventPump) -> EventHandler {
        EventHandler {
            event_pump,
            last_x: 0,
            last_y: 0,
            is_dragging: false,
            is_right_dragging: false,
        }
    }

    pub fn handle_events(&mut self, view_state: &mut ViewState, light: &mut Light) -> EventCallback {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } => return EventCallback::Resized(width as u32, height as u32),
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return EventCallback::Quit,
                Event::MouseButtonDown { mouse_btn, x, y, .. } => match mouse_btn {
                    MouseButton::Left => {
                        self.is_dragging = true;
                        self.last_x = x;
                        self.last_y = y;
                    },
                    MouseButton::Right => {
                        self.is_right_dragging = true;
                        self.last_x = x;
                        self.last_y = y;
                    },
                    _ => {}
                },
                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => {
                        self.is_dragging = false;
                    },
                    MouseButton::Right => {
                        self.is_right_dragging = false;
                    },
                    _ => {}
                },
                Event::MouseMotion { x, y, .. } => {
                    if self.is_dragging {
                        let dx = x - self.last_x;
                        let dy = y - self.last_y;
                        view_state.angle_x -= dy as f32 * 0.01;
                        view_state.angle_y += dx as f32 * 0.01;
                        self.last_x = x;
                        self.last_y = y;
                    }
                    if self.is_right_dragging {
                        let dx = x - self.last_x;
                        let dy = y - self.last_y;
                        light.x += dx as f32 * 0.5;
                        light.y -= dy as f32 * 0.5;
                        self.last_x = x;
                        self.last_y = y;
                    }
                },
                Event::MouseWheel { y, .. } => {
                    light.intensity += y as f32 * 0.1;
                    if light.intensity < 0.0 {
                        light.intensity = 0.0;
                    }
                },
                _ => {}
            }
        }
        EventCallback::Next
    }
}
