use anyhow::Result;
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Material, Mesh, Object, Texture, Vertex, UNLIT_FRAG, UNLIT_VERT,
};
use nalgebra::{Matrix4, Point3};
use std::fs::File;

struct MyApp {
    img_data: Vec<(Vec<u8>, u32)>,
    texture: Texture,
    material: Material,
    mesh: Mesh,
    time: f32,
    img_frame: u32,
}

fn read_png(path: &str) -> Result<(Vec<u8>, u32)> {
    // Read important image data
    let img = png::Decoder::new(File::open(path)?);
    let (info, mut reader) = img.read_info()?;
    assert!(info.color_type == png::ColorType::RGB);
    assert!(info.bit_depth == png::BitDepth::Eight);
    let mut img_buffer = vec![0; info.buffer_size()];
    assert_eq!(info.buffer_size(), (info.width * info.height * 3) as _);
    reader.next_frame(&mut img_buffer)?;
    Ok((img_buffer, info.width))
}

impl App for MyApp {
    const NAME: &'static str = "MyApp";

    type Args = ();

    fn new(engine: &mut dyn Engine, _args: Self::Args) -> Result<Self> {
        let img_data = (0..3)
            .map(|frame| read_png(&format!("./examples/obama{}.png", frame)))
            .collect::<Result<Vec<_>>>()?;

        let (img, width) = &img_data[0];
        let texture = engine.add_texture(img, *width)?;

        let material = engine.add_material(UNLIT_VERT, UNLIT_FRAG, DrawType::Triangles)?;

        let (vertices, indices) = rainbow_cube();
        let mesh = engine.add_mesh(&vertices, &indices)?;

        Ok(Self {
            img_frame: 0,
            img_data,
            texture,
            mesh,
            material,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        self.img_frame += 1;
        if self.img_frame % 9 == 0 {
            let (img, width) = &self.img_data[(self.img_frame % 3) as usize];
            engine.update_texture(self.texture, img, *width)?;
        }

        let transform = Matrix4::from_euler_angles(0.0, self.time, 0.0);
        let object = Object {
            material: self.material,
            mesh: self.mesh,
            texture: self.texture,
            transform,
        };
        engine.update_time_value(self.time)?;
        self.time += 0.01;
        Ok(FramePacket {
            objects: vec![object],
        })
    }
}

fn main() -> Result<()> {
    let vr = std::env::args().skip(1).next().is_some();
    launch::<MyApp>(vr, ())
}

fn rainbow_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::from_nalgebra(Point3::new(-1.0, -1.0, -1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, -1.0, -1.0), Point3::new(1.0, 0.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, 1.0, -1.0), Point3::new(1.0, 1.0, 0.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, 1.0, -1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, -1.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(1.0, -1.0, 1.0), Point3::new(1.0, 1.0, 0.0)),
        Vertex::from_nalgebra(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 1.0, 1.0)),
        Vertex::from_nalgebra(Point3::new(-1.0, 1.0, 1.0), Point3::new(1.0, 0.0, 1.0)),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    (vertices, indices)
}
