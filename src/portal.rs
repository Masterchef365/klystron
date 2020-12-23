use nalgebra::Matrix4;

pub fn portal_view_logic(eye: Matrix4<f32>, [blue, orange]: [Matrix4<f32>; 2]) -> [Matrix4<f32>; 2] {
    let orange_inv = orange.try_inverse().unwrap();
    let blue_inv = blue.try_inverse().unwrap();

    let orange = eye * orange * blue_inv;
    let blue = eye * blue * orange_inv;
    [blue, orange]
}
