use crate::core::PortalCamera;
use crate::Portal;
use nalgebra::{Matrix4, Point3};

/// Actually it should be an accumulation model, where our view matrix is concatenated and
/// unconcated by the view for each direction through the portal
struct PortalMotion {
    /// For calculating steps
    last_position: Point3<f32>,
    /// Current view matrix, concatenated with opposite portal's view each time
    /// The "Affine transformation due to portals"
    current_view: Matrix4<f32>,
}

impl PortalMotion {
    pub fn new(initial_position: Point3<f32>) -> Self {
        Self {
            last_position: initial_position,
            current_view: Matrix4::identity(),
        }
    }

    /// Returns a matrix to tranverse the eyes to relative the portal output
    pub fn update(
        &mut self,
        new_position: Point3<f32>,
        orange_portal: &Portal,
        blue_portal: &Portal,
    ) -> (Point3<f32>, Matrix4<f32>) {
        let orange_view = portal_view(orange_portal);
        let blue_view = portal_view(orange_portal);

        // Reverse towards the origin, at the offset from this portal, then move to the other
        // portal with this same offset set (Might need to mirror?)
        if portal_intersection(orange_portal, self.last_position, new_position) {
            self.last_position = Point3::from_homogeneous(
                blue_view * (orange_view.try_inverse().unwrap() * new_position.to_homogeneous()),
            )
            .unwrap();
            self.current_view *= blue_view;
        }

        if portal_intersection(blue_portal, self.last_position, new_position) {
            self.last_position = Point3::from_homogeneous(
                orange_view * (blue_view.try_inverse().unwrap() * new_position.to_homogeneous()),
            )
            .unwrap();
            self.current_view *= orange_view;
        }

        (self.last_position, self.current_view)
    }
}

fn portal_view(portal: &Portal) -> Matrix4<f32> {
    let (roll, pitch, yaw) = portal.rotation;
    Matrix4::new_translation(&portal.origin.coords) * Matrix4::from_euler_angles(roll, pitch, yaw)
}

fn portal_intersection(portal: &Portal, start: Point3<f32>, end: Point3<f32>) -> bool {
    todo!()
}

fn cross_triangle(tri: &[Point3<f32>; 3], start: Point3<f32>, end: Point3<f32>) -> bool {
    todo!()
}
