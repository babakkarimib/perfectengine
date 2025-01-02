use sdl2::{event::{Event, WindowEvent}, keyboard::Keycode, mouse::MouseButton, EventPump};
use crate::types::{event_callback::EventCallback, light::Light, view_state::ViewState};

pub struct EventHandler {
    event_pump: EventPump,
    last_x: i32,
    last_y: i32,
    drag: bool,
    r_drag: bool,
    m_drag: bool,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    r_move_forward: bool,
    r_move_backward: bool,
    r_move_left: bool,
    r_move_right: bool,
}

impl EventHandler {
    pub fn new(event_pump: EventPump) -> EventHandler {
        EventHandler {
            event_pump,
            last_x: 0,
            last_y: 0,
            drag: false,
            r_drag: false,
            m_drag: false,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            r_move_forward: false,
            r_move_backward: false,
            r_move_left: false,
            r_move_right: false,
        }
    }

    pub fn handle_events(&mut self, view_state: &mut ViewState, light: &mut Light) -> Option<EventCallback> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } => return Some(EventCallback::Resized(width as u32, height as u32)),
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return Some(EventCallback::Quit),
                Event::KeyDown { keycode: Some(key), .. } | Event::KeyUp { keycode: Some(key), .. } => {
                    let pressed = matches!(event, Event::KeyDown { .. });
                    match key {
                        Keycode::W => self.move_forward = pressed,
                        Keycode::S => self.move_backward = pressed,
                        Keycode::A => self.move_left = pressed,
                        Keycode::D => self.move_right = pressed,
                        Keycode::Up => self.r_move_forward = pressed,
                        Keycode::Down => self.r_move_backward = pressed,
                        Keycode::Left => self.r_move_left = pressed,
                        Keycode::Right => self.r_move_right = pressed,
                        _ => {}
                    }
                },
                Event::MouseButtonDown { mouse_btn, x, y, .. } => match mouse_btn {
                    MouseButton::Left => {
                        self.drag = true;
                        self.last_x = x;
                        self.last_y = y;
                    },
                    MouseButton::Right => {
                        self.r_drag = true;
                        self.last_x = x;
                        self.last_y = y;
                    },
                    MouseButton::Middle => {
                        self.m_drag = true;
                        self.last_x = x;
                        self.last_y = y;
                    },
                    _ => {}
                },
                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => self.drag = false,
                    MouseButton::Right => self.r_drag = false,
                    MouseButton::Middle => self.m_drag = false,
                    _ => {}
                },
                Event::MouseMotion { x, y, .. } => {
                    if self.drag {
                        let dx = x - self.last_x;
                        let dy = y - self.last_y;
                        view_state.angle_x += dy as f32 * 0.01 % 360.0;
                        view_state.angle_y += dx as f32 * 0.01 % 360.0;
                        self.last_x = x;
                        self.last_y = y;
                    }
                    if self.r_drag {
                        let dx = x - self.last_x;
                        let dy = y - self.last_y;
                        light.x += dx as f32 * 0.5;
                        light.y -= dy as f32 * 0.5;
                        self.last_x = x;
                        self.last_y = y;
                    }
                    if self.m_drag {
                        let dx = x - self.last_x;
                        let dy = y - self.last_y;
                        view_state.c_angle_x -= dy as f32 * 0.01 % 360.0;
                        view_state.c_angle_y -= dx as f32 * 0.01 % 360.0;
                        self.last_x = x;
                        self.last_y = y;
                    }
                },
                Event::MouseWheel { y, .. } => {
                    light.intensity += y as f32 * 0.5;
                    if light.intensity < 0.0 {
                        light.intensity = 0.0;
                    }
                },
                _ => {}
            }
        }

        if self.move_forward { view_state.camera_z -= 2.0; }
        if self.move_backward { view_state.camera_z += 2.0; }
        if self.move_left { view_state.camera_x += 2.0; }
        if self.move_right { view_state.camera_x -= 2.0; }
        if self.r_move_forward { view_state.ref_z -= 2.0; }
        if self.r_move_backward { view_state.ref_z += 2.0; }
        if self.r_move_left { view_state.ref_x -= 2.0; }
        if self.r_move_right { view_state.ref_x += 2.0; }

        None
    }
}
