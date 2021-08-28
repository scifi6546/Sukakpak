use na::Vector2;
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    time::Duration,
};
use sukakpak::{
    anyhow::Result, image, nalgebra as na, BoundFramebuffer, Context, Event, Framebuffer, Mesh,
    MeshAsset, MeshTexture, MouseButton,
};
pub struct CloneCraft {
    camera_matrix: na::Matrix4<f32>,
    triangle: Mesh,
    framebuffer: Framebuffer,
    frame_counter: f32,
    plane: Mesh,
    cube_scale: f32,
    cube_pos: na::Vector2<f32>,
    sphere: Mesh,
    num_frames: u32,
    textured_cube: Mesh,
    #[allow(dead_code)]
    mountain_tex: MeshTexture,
    #[allow(dead_code)]
    red_texture: MeshTexture,
    #[allow(dead_code)]
    blue_texture: MeshTexture,
    alt_fb_mesh: Mesh,
    alt_fb: Framebuffer,
}
impl CloneCraft {
    fn draw_rotating_cube<'a>(&self, context: &mut RefMut<Context>, pos: na::Vector2<f32>) {
        let rot = na::Matrix4::from_euler_angles(
            self.frame_counter / 335.0,
            self.frame_counter / 107.2,
            0.0,
        );
        let mat = na::Matrix4::new_translation(&na::Vector3::new(pos.x, -1.0 * pos.y, 0.0))
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(self.cube_scale, self.cube_scale, self.cube_scale),
                &na::Point3::new(0.0, 0.0, 0.0),
            )
            * rot
            * na::Matrix4::new_translation(&na::Vector3::new(-0.5, -0.5, 0.0));
        context
            .draw_mesh(to_slice(&mat), &self.sphere)
            .expect("failed to draw triangle");
    }
    fn draw_cube(&self, context: &mut RefMut<Context>, scale: f32) -> Result<()> {
        let mat = self.camera_matrix
            * na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0))
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(scale, scale, scale),
                &na::Point3::new(0.0, 0.0, 0.0),
            );
        context.draw_mesh(to_slice(&mat), &self.textured_cube)
    }
    fn draw_fb_plane(
        &self,
        context: &mut RefMut<Context>,
        bound_fb: &BoundFramebuffer,
    ) -> Result<()> {
        let mat = na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, 0.2))
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(0.2, 0.2, 0.5),
                &na::Point3::new(0.0, 0.0, 0.0),
            );
        context.bind_framebuffer(&BoundFramebuffer::UserFramebuffer(self.alt_fb))?;
        self.draw_cube(context, 1.5)?;
        context.bind_framebuffer(bound_fb)?;
        context.draw_mesh(to_slice(&mat), &self.alt_fb_mesh)?;

        Ok(())
    }
}
fn to_slice(mat: &na::Matrix4<f32>) -> Vec<u8> {
    mat.as_slice()
        .iter()
        .map(|f| f.to_ne_bytes())
        .flatten()
        .collect()
}
const CUBE_DIMENSIONS: usize = 1;
impl sukakpak::Renderable for CloneCraft {
    fn init(context: Rc<RefCell<Context>>) -> Self {
        let mut ctx_ref = context.borrow_mut();
        ctx_ref
            .load_shader("shaders/test", "test")
            .expect("failed to load shader");
        let image = image::ImageBuffer::from_pixel(100, 100, image::Rgba([255, 0, 0, 0]));
        let red_texture = ctx_ref
            .build_texture(&image)
            .expect("failed to create image");
        let blue_texture = ctx_ref
            .build_texture(&image::ImageBuffer::from_pixel(
                100,
                100,
                image::Rgba([0, 0, 255, 0]),
            ))
            .expect("failed to build texture");
        let sphere_obj = MeshAsset::from_obj("sphere.obj").expect("");
        let sphere = ctx_ref.build_mesh(sphere_obj, red_texture);
        let mountain_tex = ctx_ref
            .build_texture(
                &image::io::Reader::open("./assets/mtn.JPG")
                    .expect("failed to open jpeg")
                    .decode()
                    .expect("failed to decode")
                    .to_rgba8(),
            )
            .expect("failed to build texture");
        let textured_cube = ctx_ref.build_mesh(MeshAsset::new_cube(), mountain_tex);
        let triangle = ctx_ref.build_mesh(MeshAsset::new_cube(), red_texture);
        let delete = ctx_ref.build_mesh(MeshAsset::new_cube(), red_texture);
        ctx_ref.delete_mesh(delete).expect("failed to delete");
        let delete = ctx_ref
            .build_texture(&image)
            .expect("failed to create image");
        ctx_ref.delete_texture(delete).expect("failed to delete");
        let framebuffer = ctx_ref
            .build_framebuffer(na::Vector2::new(300, 300))
            .expect("failed to build frame buffer");
        let alt_fb = ctx_ref
            .build_framebuffer(na::Vector2::new(1000, 1000))
            .expect("failed to build frame buffer");
        let alt_fb_mesh =
            ctx_ref.build_mesh(MeshAsset::new_plane(), MeshTexture::Framebuffer(alt_fb));
        ctx_ref
            .bind_shader(&BoundFramebuffer::UserFramebuffer(framebuffer), "alt")
            .expect("failed to bind");
        let mut plane = ctx_ref.build_mesh(MeshAsset::new_plane(), red_texture);
        plane.bind_framebuffer(framebuffer);
        let camera_matrix =
            *na::Perspective3::new(1.0, std::f32::consts::PI as f32 / 4.0, 1.0, 1000.0).as_matrix();
        Self {
            camera_matrix,
            triangle,
            alt_fb,
            sphere,
            red_texture,
            frame_counter: 0.0,
            cube_scale: 0.2,
            blue_texture,
            framebuffer,
            textured_cube,
            mountain_tex,
            alt_fb_mesh,
            cube_pos: Vector2::new(0.0, 0.0),
            num_frames: 0,
            plane,
        }
    }
    fn render_frame(
        &mut self,
        events: &[Event],
        context: Rc<RefCell<Context>>,
        delta_time: Duration,
    ) {
        for e in events.iter() {
            match e {
                Event::MouseMoved { normalized, .. } => self.cube_pos = *normalized,
                Event::MouseDown { button } => {
                    if button == &MouseButton::Left {
                        self.triangle.bind_texture(self.blue_texture)
                    }
                }
                Event::MouseUp { button } => {
                    if button == &MouseButton::Left {
                        self.triangle.bind_texture(self.red_texture)
                    }
                }
                Event::ScrollContinue { delta } => {
                    self.cube_scale += delta.delta.y * delta_time.as_secs_f32() * 0.01;
                }
                _ => (),
            }
        }
        let mut ctx_ref = context.borrow_mut();
        if self.num_frames >= 10 {
            let delete_cube = ctx_ref.build_mesh(MeshAsset::new_cube(), self.mountain_tex);
            let mat = self.camera_matrix
                * na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0))
                * na::Matrix4::new_nonuniform_scaling_wrt_point(
                    &na::Vector3::new(0.1, 0.1, 0.1),
                    &na::Point3::new(0.0, 0.0, 0.0),
                );
            ctx_ref.draw_mesh(to_slice(&mat), &delete_cube);
            ctx_ref.delete_mesh(delete_cube);
        }
        ctx_ref
            .bind_framebuffer(&BoundFramebuffer::UserFramebuffer(self.framebuffer))
            .expect("failed to bind");
        self.draw_cube(&mut ctx_ref, 1.0).expect("failed to draw");
        self.draw_fb_plane(
            &mut ctx_ref,
            &BoundFramebuffer::UserFramebuffer(self.framebuffer),
        )
        .expect("failed to draw");
        self.draw_rotating_cube(&mut ctx_ref, self.cube_pos);
        ctx_ref
            .bind_framebuffer(&BoundFramebuffer::ScreenFramebuffer)
            .expect("failed to bind");
        self.draw_rotating_cube(&mut ctx_ref, self.cube_pos);
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
                    ctx_ref
                        .draw_mesh(to_slice(&mat), &new_mesh)
                        .expect("failed to draw triangle");
                }
            }
        }
        self.draw_fb_plane(&mut ctx_ref, &BoundFramebuffer::ScreenFramebuffer)
            .expect("failed to draw");
        self.draw_rotating_cube(&mut ctx_ref, self.cube_pos);
        let plane_mat = self.camera_matrix
            * transorm_mat
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(2.0, 2.0, 4.0),
                &na::Point3::new(0.0, 0.0, 0.0),
            )
            * na::Matrix4::new_translation(&na::Vector3::new(0.5, 0.5, 0.0));
        ctx_ref
            .draw_mesh(to_slice(&plane_mat), &self.plane)
            .expect("failed to draw");
        self.frame_counter += delta_time.as_secs_f32();
        self.num_frames += 1;
    }
}
