use sukakpak::{
    nalgebra as na, BoundFramebuffer, ContextChild, Event, Framebuffer, Mesh, MeshAsset, Texture,
};
pub struct CloneCraft {
    camera_matrix: na::Matrix4<f32>,
    triangle: Mesh,
    framebuffer: Framebuffer,
    frame_counter: u64,
    plane: Mesh,
    #[allow(dead_code)]
    red_texture: Texture,
    #[allow(dead_code)]
    blue_texture: Texture,
}
impl CloneCraft {
    fn draw_rotating_cube<'a>(&self, context: &mut ContextChild<'a>) {
        let transorm_mat = na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0));

        let rot = na::Matrix4::from_euler_angles(
            self.frame_counter as f32 / 123.0,
            self.frame_counter as f32 / 100.0,
            0.0,
        );
        let mat = self.camera_matrix
            * transorm_mat
            * rot
            * na::Matrix4::new_translation(&na::Vector3::new(-0.5, -0.5, -0.5));
        context
            .draw_mesh(to_slice(&mat), &self.triangle)
            .expect("failed to draw triangle");
    }
}
fn to_slice(mat: &na::Matrix4<f32>) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            mat.as_ptr() as *const u8,
            std::mem::size_of::<na::Matrix4<f32>>(),
        )
    }
}
const CUBE_DIMENSIONS: usize = 10;
impl sukakpak::Renderable for CloneCraft {
    fn init<'a>(context: &mut ContextChild<'a>) -> Self {
        let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
        let red_texture = context
            .build_texture(&image)
            .expect("failed to create image");
        let blue_texture = context
            .build_texture(&image::ImageBuffer::from_pixel(
                100,
                100,
                image::Rgba([0, 0, 255, 0]),
            ))
            .expect("failed to build texture");

        let triangle = context.build_meshes(MeshAsset::new_cube(), red_texture);
        let framebuffer = context
            .build_framebuffer(na::Vector2::new(300, 300))
            .expect("failed to build frame buffer");
        context
            .bind_shader(&BoundFramebuffer::UserFramebuffer(framebuffer), "alt")
            .expect("failed to bind");
        let mut plane = context.build_meshes(MeshAsset::new_plane(), red_texture);
        plane.bind_framebuffer(framebuffer);
        let camera_matrix = *na::Perspective3::new(1.0, 3.14 / 4.0, 1.0, 1000.0).as_matrix();
        Self {
            camera_matrix,
            triangle,
            red_texture,
            frame_counter: 0,
            blue_texture,
            framebuffer,
            plane,
        }
    }
    fn render_frame<'a>(&mut self, _events: &[Event], context: &mut ContextChild<'a>) {
        context
            .bind_framebuffer(&BoundFramebuffer::UserFramebuffer(self.framebuffer))
            .expect("failed to bind");
        self.draw_rotating_cube(context);
        context
            .bind_framebuffer(&BoundFramebuffer::ScreenFramebuffer)
            .expect("failed to bind");

        let transorm_mat = na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0));

        let rot = na::Matrix4::from_euler_angles(
            self.frame_counter as f32 / 1213.0,
            self.frame_counter as f32 / 1000.0,
            0.0,
        );
        for x in 0..CUBE_DIMENSIONS {
            for y in 0..CUBE_DIMENSIONS {
                for z in 0..CUBE_DIMENSIONS {
                    let transorm_mat = na::Matrix4::new_translation(&na::Vector3::new(
                        x as f32,
                        y as f32,
                        z as f32 - 100.0,
                    ));
                    let mat = self.camera_matrix
                        * transorm_mat
                        * rot
                        * na::Matrix4::new_translation(&na::Vector3::new(-0.5, -0.5, -0.5));
                    let mut new_mesh = self.triangle.clone();
                    new_mesh.bind_texture(self.blue_texture);
                    if y % 2 == 0 {
                        new_mesh.bind_texture(self.red_texture);
                    }

                    context
                        .draw_mesh(to_slice(&mat), &new_mesh)
                        .expect("failed to draw triangle");
                }
            }
        }
        //self.draw_rotating_cube(context);
        let plane_mat = self.camera_matrix
            * transorm_mat
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(2.0, 2.0, 4.0),
                &na::Point3::new(0.0, 0.0, 0.0),
            )
            * na::Matrix4::new_translation(&na::Vector3::new(0.5, 0.5, 0.0));
        context
            .draw_mesh(to_slice(&plane_mat), &self.plane)
            .expect("failed to draw");
        self.frame_counter += 1;
    }
}
