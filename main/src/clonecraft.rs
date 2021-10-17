use na::Vector2;
use na::Vector3;
use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    time::Duration,
};
use sukakpak::{
    anyhow::Result, image, nalgebra as na, Bindable, Context, ContextTrait, DrawableTexture, Event,
    Framebuffer, Mesh, MeshAsset, MouseButton, Texture,
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
    mountain_tex: Texture,
    #[allow(dead_code)]
    red_texture: Texture,
    #[allow(dead_code)]
    blue_texture: Texture,
    alt_fb_mesh: Mesh,
    alt_fb: Framebuffer,
}
impl CloneCraft {
    fn draw_rotating_cube<'a>(&self, mut context: Context, pos: na::Vector2<f32>) {
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
    fn draw_cube(&self, mut context: Context, scale: f32) -> Result<()> {
        let mat = self.camera_matrix
            * na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0))
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(scale, scale, scale),
                &na::Point3::new(0.0, 0.0, 0.0),
            );
        context.draw_mesh(to_slice(&mat), &self.textured_cube)
    }
    fn draw_fb_plane(&self, mut context: Context, bound_fb: Bindable) -> Result<()> {
        let mat = na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, 0.2))
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(0.2, 0.2, 0.5),
                &na::Point3::new(0.0, 0.0, 0.0),
            );
        let mat = na::Matrix4::<f32>::identity()
            * na::Matrix4::new_translation(&na::Vector3::new(-1.0, -1.0, 0.2))
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &Vector3::new(1.0, 1.0, 1.0),
                &na::Point3::new(0.0, 0.0, 0.0),
            );
        context.bind_framebuffer(Bindable::UserFramebuffer(&self.alt_fb))?;
        self.draw_cube(context.clone(), 1.5)?;
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
    fn init(mut context: Context) -> Self {
        //context
        //    .load_shader("shaders/test", "test")
        //    .expect("failed to load shader");
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
        let sphere_obj = MeshAsset::from_obj("sphere.obj").expect("");
        let sphere = context
            .build_mesh(sphere_obj, DrawableTexture::Texture(&red_texture))
            .expect("failed to build circle");
        let mountain_tex = context
            .build_texture(
                &image::io::Reader::open("./assets/mtn.JPG")
                    .expect("failed to open jpeg")
                    .decode()
                    .expect("failed to decode")
                    .to_rgba8(),
            )
            .expect("failed to build texture");
        let textured_cube = context
            .build_mesh(
                MeshAsset::new_cube(),
                DrawableTexture::Texture(&mountain_tex),
            )
            .expect("failed to build cube");
        let triangle = context
            .build_mesh(
                MeshAsset::new_cube(),
                DrawableTexture::Texture(&red_texture),
            )
            .expect("failed to build mesh");
        {
            let delete = context
                .build_mesh(
                    MeshAsset::new_cube(),
                    DrawableTexture::Texture(&red_texture),
                )
                .expect("failed to build mesh");
        }
        {
            let delete = context
                .build_texture(&image)
                .expect("failed to create image");
        }
        let framebuffer = context
            .build_framebuffer(na::Vector2::new(300, 300))
            .expect("failed to build frame buffer");
        let alt_fb = context
            .build_framebuffer(na::Vector2::new(1000, 1000))
            .expect("failed to build frame buffer");
        let alt_fb_mesh = context
            .build_mesh(
                MeshAsset::new_plane(),
                DrawableTexture::Framebuffer(&alt_fb),
            )
            .expect("failed to build fb mesh");
        context
            .load_shader(include_str!("../v2_test.ass_spv"), "v2")
            .expect("failed to load shader");
        context
            .bind_shader(Bindable::UserFramebuffer(&framebuffer), "v2")
            .expect("failed to bind");
        //  context
        //      .bind_shader(Bindable::ScreenFramebuffer, "v2")
        //      .expect("failed to bind");
        let mut plane = context
            .build_mesh(
                MeshAsset::new_plane(),
                DrawableTexture::Texture(&red_texture),
            )
            .expect("failed to build plane");
        context
            .bind_texture(&mut plane, DrawableTexture::Framebuffer(&framebuffer))
            .expect("failed to draw plane framebuffer");
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
    fn render_frame(&mut self, events: &[Event], mut context: Context, delta_time: Duration) {
        for e in events.iter() {
            match e {
                Event::MouseMoved { normalized, .. } => self.cube_pos = *normalized,
                Event::MouseDown { button } => {
                    if button == &MouseButton::Left {
                        context
                            .bind_texture(
                                &mut self.triangle,
                                DrawableTexture::Texture(&self.blue_texture),
                            )
                            .expect("failed to bind texture");
                    }
                }
                Event::MouseUp { button } => {
                    if button == &MouseButton::Left {
                        context
                            .bind_texture(
                                &mut self.triangle,
                                DrawableTexture::Texture(&self.red_texture),
                            )
                            .expect("failed to bind texture");
                    }
                }
                Event::ScrollContinue { delta } => {
                    self.cube_scale += delta.delta.y * delta_time.as_secs_f32() * 0.01;
                }
                _ => (),
            }
        }
        {
            let delete_cube = context
                .build_mesh(
                    MeshAsset::new_cube(),
                    DrawableTexture::Texture(&self.mountain_tex),
                )
                .expect("failed to build");
            let mat = self.camera_matrix
                * na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0))
                * na::Matrix4::new_nonuniform_scaling_wrt_point(
                    &na::Vector3::new(0.1, 0.1, 0.1),
                    &na::Point3::new(0.0, 0.0, 0.0),
                );
            context
                .draw_mesh(to_slice(&mat), &delete_cube)
                .expect("failed to draw");
        }
        context
            .bind_framebuffer(Bindable::UserFramebuffer(&self.framebuffer))
            .expect("failed to bind");
        self.draw_cube(context.clone(), 1.0)
            .expect("failed to draw");
        self.draw_fb_plane(
            context.clone(),
            Bindable::UserFramebuffer(&self.framebuffer),
        )
        .expect("failed to draw");
        self.draw_rotating_cube(context.clone(), self.cube_pos);
        context
            .bind_framebuffer(Bindable::ScreenFramebuffer)
            .expect("failed to bind");
        self.draw_rotating_cube(context.clone(), self.cube_pos);
        let transorm_mat = na::Matrix4::new_translation(&na::Vector3::new(0.0, 0.0, -10.0));

        let rot = na::Matrix4::from_euler_angles(
            self.frame_counter as f32 / 1213.0,
            self.frame_counter as f32 / 1000.0,
            0.0,
        );
        self.draw_fb_plane(context.clone(), Bindable::ScreenFramebuffer)
            .expect("failed to draw");
        self.draw_rotating_cube(context.clone(), self.cube_pos);
        let plane_mat = self.camera_matrix
            * transorm_mat
            * na::Matrix4::new_nonuniform_scaling_wrt_point(
                &na::Vector3::new(2.0, 2.0, 4.0),
                &na::Point3::new(0.0, 0.0, 0.0),
            )
            * na::Matrix4::new_translation(&na::Vector3::new(0.5, 0.5, 0.0));
        let plane_mat = na::Matrix4::identity();
        context
            .draw_mesh(to_slice(&plane_mat), &self.plane)
            .expect("failed to draw");
        self.frame_counter += delta_time.as_secs_f32();
        self.num_frames += 1;
    }
}
