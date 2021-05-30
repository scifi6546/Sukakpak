pub use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};

mod graphics;

use graphics::{Context, Vertex};
use nalgebra::{Matrix4, Perspective3, Vector2, Vector3};

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
        &[
            image::open("./texture.jpeg").unwrap().into_rgba8(),
            image::open("./textures/grass.png").unwrap().into_rgba8(),
        ],
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
    let grass_cube = context.new_mesh(
        textures[1],
        vec![
            //face 0
            Vertex {
                position: Vector3::new(1.0, 0.0, 1.0),
                uv: Vector2::new(2.0 / 6.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(2.0 / 6.0, 1.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                uv: Vector2::new(1.0 / 6.0, 1.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                uv: Vector2::new(1.0 / 6.0, 0.0),
            },
            //face 1
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(3.0 / 6.0, 0.0),
            },
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(3.0 / 6.0, 1.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                uv: Vector2::new(2.0 / 6.0, 0.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(2.0 / 6.0, 1.0),
            },
            //Face 2
            //8
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(4.0 / 6.0, 0.0),
            },
            //9
            Vertex {
                position: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(4.0 / 6.0, 1.0),
            },
            //10
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(3.0 / 6.0, 0.0),
            },
            //11
            Vertex {
                position: Vector3::new(0.0, 1.0, 1.0),
                uv: Vector2::new(3.0 / 6.0, 1.0),
            },
            //Face 3
            //12
            Vertex {
                position: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(5.0 / 6.0, 0.0),
            },
            //13
            Vertex {
                position: Vector3::new(1.0, 0.0, 1.0),
                uv: Vector2::new(5.0 / 6.0, 1.0),
            },
            //14
            Vertex {
                position: Vector3::new(0.0, 1.0, 1.0),
                uv: Vector2::new(4.0 / 6.0, 0.0),
            },
            //15
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                uv: Vector2::new(4.0 / 6.0, 1.0),
            },
            //face 4
            //16
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            //17
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                uv: Vector2::new(1.0 / 6.0, 0.0),
            },
            //18
            Vertex {
                position: Vector3::new(0.0, 1.0, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
            //19
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(1.0 / 6.0, 1.0),
            },
            //face 5
            //20
            Vertex {
                position: Vector3::new(1.0, 0.0, 1.0),
                uv: Vector2::new(5.0 / 6.0, 0.0),
            },
            //21
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(5.0 / 6.0, 1.0),
            },
            //22
            Vertex {
                position: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(6.0 / 6.0, 0.0),
            },
            //23
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(6.0 / 6.0, 1.0),
            },
        ],
        vec![
            [0, 1, 2],
            [0, 2, 3],
            [5, 6, 4],
            [5, 7, 6],
            [8, 9, 10],
            [10, 9, 11],
            [12, 13, 15],
            [12, 15, 14],
            [16, 17, 19],
            [16, 19, 18],
            [20, 23, 21],
            [20, 22, 23],
        ]
        .iter()
        .flatten()
        .copied()
        .collect(),
    );
    let cube = context.new_mesh(
        textures[0],
        vec![
            Vertex {
                position: Vector3::new(1.0, 0.0, 1.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 0.0, 1.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(0.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 1.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ],
        vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 3, 0],
            [4, 7, 3],
            [3, 6, 2],
            [3, 7, 6],
            [4, 5, 6],
            [4, 6, 7],
            [5, 1, 2],
            [5, 2, 6],
            [0, 1, 5],
            [0, 5, 4],
        ]
        .iter()
        .map(|[a, b, c]| [c, b, a])
        .flatten()
        .copied()
        .collect(),
    );
    let mut counter = 0;

    event_loop.run(move |event, _, control_flow| {
        counter += 1;
        let rotation = (counter as f32) / 1000.0;
        let perspective = *Perspective3::new(1.0, 3.14 / 4.0, 0.1, 100.0).as_matrix();
        let cubes_mat = (0..4)
            .map(|x| {
                (0..4).map(move |y| {
                    (0..4).map(move |z| {
                        perspective
                            * Matrix4::new_translation(&Vector3::new(0.0, 0.0, -10.0))
                            * Matrix4::from_euler_angles(-0.7 * rotation, 0.0, 0.0)
                            * Matrix4::new_scaling(0.2)
                            * Matrix4::new_translation(&Vector3::new(x as f32, y as f32, z as f32))
                    })
                })
            })
            .flatten()
            .flatten()
            .collect::<Vec<_>>();
        let mat1: Matrix4<f32> = perspective
            * Matrix4::new_translation(&Vector3::new(0.5, 0.0, -6.0))
            * Matrix4::from_euler_angles(rotation, 0.0, 0.0);
        let mat2: Matrix4<f32> = perspective
            * Matrix4::new_translation(&Vector3::new(-0.5, 0.0, -3.0))
            * Matrix4::from_euler_angles(-1.0 * rotation, 0.0, 0.0);
        let cube_mat: Matrix4<f32> = perspective
            * Matrix4::new_translation(&Vector3::new(0.0, 0.0, -2.0))
            * Matrix4::new_scaling(0.1)
            * Matrix4::from_euler_angles(0.0, -0.7 * rotation, 0.0);
        let grass_cube_mat: Matrix4<f32> = perspective
            * Matrix4::new_translation(&Vector3::new(0.0, 0.0, -1.3))
            * Matrix4::new_scaling(0.1)
            * Matrix4::from_euler_angles(-0.7 * rotation, 0.0, 0.0);
        let mats = cubes_mat
            .iter()
            .map(|mat| (grass_cube, mat.as_ptr() as *const std::ffi::c_void))
            .collect::<Vec<_>>();
        //mats.push((mesh, mat1.as_ptr() as *const std::ffi::c_void));
        //mats.push((mesh, mat2.as_ptr() as *const std::ffi::c_void));
        //mats.push((cube, cube_mat.as_ptr() as *const std::ffi::c_void));
        context.render_frame(&mats);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
