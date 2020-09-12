use super::camera::Camera;
use nalgebra::{Vector4, Vector3, Matrix4};
use winit::event::{WindowEvent, MouseButton, MouseScrollDelta, ElementState};
use winit::dpi::{PhysicalPosition, PhysicalSize};

pub struct MouseCamera {
    pub(crate) internal: Camera,
    sensitivity: f32,
    last_mouse_position: Option<(f64, f64)>,
    left_is_clicked: bool,
    right_is_clicked: bool,
}

impl MouseCamera {
    pub fn new(internal: Camera, sensitivity: f32) -> Self {
        Self {
            internal,
            sensitivity,
            last_mouse_position: None,
            left_is_clicked: false,
            right_is_clicked: false,
        }
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.internal.matrix()
    }

    pub fn handle_events(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let PhysicalPosition { x, y } = position;
                if let Some((last_x, last_y)) = self.last_mouse_position {
                    let x_delta = (*x - last_x) as f32;
                    let y_delta = (*y - last_y) as f32;
                    if self.left_is_clicked {
                        self.mouse_pivot(x_delta, y_delta);
                    } else if self.right_is_clicked {
                        self.mouse_pan(x_delta, y_delta);
                    }
                }
                self.last_mouse_position = Some((*x, *y));
            }
            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => {
                    self.left_is_clicked = *state == ElementState::Pressed
                }
                MouseButton::Right => {
                    self.right_is_clicked = *state == ElementState::Pressed
                }
                _ => (),
            },
            WindowEvent::Resized(size) => {
                let PhysicalSize { width, height } = size;
                self.internal.aspect = *width as f32 / *height as f32;
            },
            WindowEvent::MouseWheel {delta, ..} => {
                if let MouseScrollDelta::LineDelta(_x, y) = delta {
                    self.internal.distance += y * -0.05;
                    if self.internal.distance <= 0.01 {
                        self.internal.distance = 0.01;
                    }
                }
            },
            _ => (),
        }
    }

    fn mouse_pivot(&mut self, delta_x: f32, delta_y: f32) {
        self.internal.yaw += delta_x * self.sensitivity;
        self.internal.pitch += delta_y * self.sensitivity;
    }

    fn mouse_pan(&mut self, delta_x: f32, delta_y: f32) {
        let view_inv = self.internal.view().try_inverse().unwrap();
        let delta = Vector4::new((delta_x as f32) * self.internal.distance, (-delta_y as f32) * self.internal.distance, 0.0, 0.0);
        self.internal.pivot += (view_inv * delta).xyz() * self.sensitivity;
    }

    pub fn camera(&self) -> &Camera {
        &self.internal
    }
}
