use nalgebra::{Matrix4, Point3, Vector3};

pub struct Camera {
    pub pivot: Point3<f32>,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
    pub clipping: (f32, f32),
}

impl Camera {
    pub fn matrix(&self) -> Matrix4<f32> {
        let perspective = Matrix4::new_perspective(self.aspect, self.fov, self.clipping.0, self.clipping.1);
        perspective * self.view()
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_lh(
            &(self.pivot + self.eye()),
            &self.pivot,
            &Vector3::new(0.0, 1.0, 0.0),
        )
    }

    pub fn eye(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos().abs(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos().abs(),
        ) * self.distance
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pivot: Point3::origin(),
            distance: 5.0,
            yaw: 1.0,
            pitch: 1.0,
            fov: 45.0f32.to_radians(),
            aspect: 1.0,
            clipping: (0.1, 1000.0),
        }
    }
}
