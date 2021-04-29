pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

mod graphics;

use graphics::Context;

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    println!("building context");
    let mut context = Context::new("Hello Context", &event_loop, 1000, 1000);

    event_loop.run(move |event, _, control_flow| {
        context.render_frame();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
