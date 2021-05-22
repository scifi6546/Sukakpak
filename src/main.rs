pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

mod graphics;

use graphics::{Context, Vertex};
use nalgebra::{Matrix4, Vector2, Vector3};

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    println!("building context");
    let (mut context, textures) = Context::new(
        "Hello Context",
        &event_loop,
        1000,
        1000,
        &[image::open("./texture.jpeg").unwrap().into_rgba8()],
    );
    let mesh = context.new_mesh(
        textures[0],
        vec![
            Vertex {
                position: Vector3::new(-0.5, -0.5, 0.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(0.5, -0.5, 0.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(0.0, 0.5, 0.0),
                uv: Vector2::new(0.5, 1.0),
            },
        ],
        vec![0, 1, 2],
    );

    event_loop.run(move |event, _, control_flow| {
        let data: Matrix4<f32> = Matrix4::identity();
        context.render_frame(&mesh, data.as_ptr() as *const std::ffi::c_void);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
