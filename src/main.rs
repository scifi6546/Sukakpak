pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

mod graphics;

use graphics::{Context, Vertex};
use nalgebra::{Vector2, Vector3};

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    println!("building context");
    let mut context = Context::new("Hello Context", &event_loop, 1000, 1000);
    let image = image::open("./texture.jpeg").unwrap().into_rgba8();
    let texture = context.new_texture(image);
    let mesh = context.new_mesh(
        texture,
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
    );

    event_loop.run(move |event, _, control_flow| {
        context.render_frame(&mesh);
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
