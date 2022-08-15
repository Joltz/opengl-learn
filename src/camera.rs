use cgmath::{InnerSpace, Matrix4, Point3, Vector3};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use std::f32::consts::PI;

const YAW: f32 = -90.0;
const PITCH: f32 = 0.0;
const SPEED: f32 = 5.0;
const SENSITIVITY: f32 = 0.2;

pub struct Camera {
    position: Point3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
    worldup: Vector3<f32>,
    yaw: f32,
    pitch: f32,
    movement_speed: f32,
    mouse_sensitivity: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Point3::<f32> { x: 0.0, y: 0.0, z: 0.0 },
            front: Vector3::<f32> { x: 0.0, y: 0.0, z: -1.0 },
            up: Vector3::<f32> { x: 0.0, y: 1.0, z: 0.0 },
            right: Vector3::<f32> { x: 0.0, y: 0.0, z: 0.0 }, // Calculate this in the new function
            worldup: Vector3::<f32> { x: 0.0, y: 1.0, z: 0.0 },
            yaw: YAW,
            pitch: PITCH,
            movement_speed: SPEED,
            mouse_sensitivity: SENSITIVITY,
        }
    }
}

impl Camera {
    pub fn new(position: Point3<f32>, up: Vector3<f32>, front: Vector3<f32>) -> Self {
        Self {
            position: position,
            front: front,
            up: up,
            ..Default::default()
        }
    }

    pub fn process_movement(&mut self, direction: Vector3<f32>, deltatime: f32) {
        let velocity = deltatime * self.movement_speed;
        if direction.z == 1.0 {
            self.position += self.front * velocity
        }
        if direction.z == -1.0 {
            self.position -= self.front * velocity
        }
        if direction.x == -1.0 {
            self.position += self.right * velocity
        }
        if direction.x == 1.0 {
            self.position -= self.right * velocity
        }
    }

    pub fn process_rotation(&mut self, position: PhysicalPosition<f64>, windowsize: PhysicalSize<u32>, deltatime: f32) {
        let pos = PhysicalPosition::<f32> {
            // Cast to f32
            x: position.x as f32,
            y: position.y as f32,
        };

        let xoffset = (pos.x - (windowsize.width as f32 / 2.0)) * self.mouse_sensitivity * deltatime; // Mouse is locked to screen center
        let yoffset = (pos.y - (windowsize.height as f32 / 2.0)) * self.mouse_sensitivity * deltatime;

        self.yaw += xoffset;
        self.pitch = (self.pitch - yoffset).clamp(0.499 * -PI, 0.499 * PI);

        self.update_camera_vectors();
    }

    fn update_camera_vectors(&mut self) {
        self.front.x = self.yaw.cos() * self.pitch.cos();
        self.front.y = self.pitch.sin();
        self.front.z = self.yaw.sin() * self.pitch.cos();
        self.front = self.front.normalize();

        self.right = self.front.cross(self.worldup).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn get_view_matrix(&mut self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.position + self.front, self.up)
    }
}
