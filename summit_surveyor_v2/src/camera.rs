use asset_manager::{AssetHandle, AssetManager};
use legion::systems::CommandBuffer;
use legion::*;
use std::f32;
use sukakpak::{
    image::{Rgba, RgbaImage},
    nalgebra::{Matrix4, Point3, Vector2, Vector3, Vector4},
    Context, DrawableTexture,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    position: Vector3<f32>,
    scale: Vector3<f32>,
    pitch: f32,
    yaw: f32,
    roll: f32,
}
impl Transform {
    /// Builds transform matrix for transform
    pub fn mat(&self) -> Matrix4<f32> {
        let rotation = Matrix4::from_euler_angles(self.roll, self.pitch, self.yaw);
        let scaling: Matrix4<f32> = Matrix4::new_nonuniform_scaling(&self.scale);
        self.get_translate_mat() * rotation * scaling
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.mat()
            .as_slice()
            .iter()
            .map(|f| f.to_ne_bytes())
            .flatten()
            .collect()
    }
    /// Gets scaling of trasform
    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale
    }
    /// adds scale to transform
    pub fn set_scale(self, scale: Vector3<f32>) -> Self {
        Self {
            position: self.position,
            scale,
            pitch: self.pitch,
            yaw: self.yaw,
            roll: self.roll,
        }
    }
    pub fn set_translation(self, translation: Vector3<f32>) -> Self {
        Self {
            scale: self.scale,
            pitch: self.pitch,
            position: translation,
            yaw: self.yaw,
            roll: self.roll,
        }
    }
    pub fn set_yaw(self, yaw: f32) -> Self {
        Self {
            scale: self.scale,
            pitch: self.pitch,
            position: self.position,
            yaw,
            roll: self.roll,
        }
    }
    /// Translates the transform by given delta
    pub fn translate(self, delta: Vector3<f32>) -> Self {
        Self {
            scale: self.scale,
            pitch: self.pitch,
            position: self.position + delta,
            yaw: self.yaw,
            roll: self.roll,
        }
    }
    pub fn get_translate_mat(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.position)
    }
    pub fn get_translation(&self) -> Vector3<f32> {
        self.position
    }
}
impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\tposition: <{}, {}, {}>\n\tscale: <{}, {}, {}>\n\tpitch: {}\n\tyaw: {}\n\troll: {}\n}}",
            self.position.x,
            self.position.y,
            self.position.z,
            self.scale.x,
            self.scale.y,
            self.scale.z,
            self.pitch,
            self.yaw,
            self.roll
        )
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
        }
    }
}
/// Ray Used for raycasting
pub struct Ray {
    /// must be a unit vector
    pub direction: Vector3<f32>,
    pub origin: Point3<f32>,
}
impl std::fmt::Display for Ray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\n\tdirection: {},\n\t origin: {}\n}}",
            self.direction, self.origin
        )
    }
}
#[derive(Debug, Clone, Copy)]
pub struct CameraInfo {
    pub fov: f32,
    pub aspect_ratio: f32,
}
pub trait Camera: Send {
    fn get_camera_info(&self) -> CameraInfo;
    fn get_projection_mat(&self) -> Matrix4<f32>;
    fn get_view_mat(&self) -> Matrix4<f32>;
    fn get_mat(&self, transform: &Transform) -> Matrix4<f32> {
        self.get_projection_mat() * self.get_view_mat() * transform.mat()
    }
    /// Gets data for shader with model transform applied
    fn to_vec(&self, transform: &Transform) -> Vec<u8> {
        self.get_mat(transform)
            .as_slice()
            .iter()
            .map(|f| f.to_ne_bytes())
            .flatten()
            .collect()
    }
    /// moves by amount in x axis, usually triggered by a,d keys on keyboard
    fn move_x(&mut self, delta: f32);
    /// moves my amount in y axis. Usually triggered by w,s keys on keyboard
    fn move_z(&mut self, delta: f32);
    /// rotates by delta. Usually triggered by mouse x axis
    fn rotate_x(&mut self, delta: f32);
    /// rotates by delta. Usually triggered by mouse y axis
    fn rotate_y(&mut self, delta: f32);

    /// updates scroll. Usually triggered by scroll
    fn update_zoom(&mut self, delta: f32);
    /// casts ray from Camera, mouse coordinates are normalized (-1,1) is the bottom left, (1,1) is
    /// the top right
    fn cast_ray(&self) -> Ray;
    fn get_origin(&self) -> Vector3<f32>;
    /// Casts ray through mouse axis
    fn cast_mouse_ray(&self, mouse_pos: Vector2<f32>) -> Ray {
        let mat = self.get_view_mat().try_inverse().unwrap();
        let origin = self.get_origin();
        let direction = mat * Vector4::new(mouse_pos.x, mouse_pos.y, 1.0, 1.0);
        let direction = (Vector3::new(direction.x, direction.y, direction.z) - origin).normalize();
        let origin = Point3::new(origin.x, origin.y, origin.z);
        Ray { origin, direction }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FPSCamera {
    position: Vector3<f32>,
    pitch: f32,
    yaw: f32,
    roll: f32,
    fov: f32,
    aspect_ratio: f32,
    near_clip: f32,
    far_clip: f32,
}
impl Default for FPSCamera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            fov: f32::consts::PI / 4.0,
            aspect_ratio: 1.0,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
}
impl FPSCamera {
    pub fn set_translation(mut self, translation: Vector3<f32>) -> Self {
        self.position = translation;
        self
    }
    pub fn translate(mut self, translation: Vector3<f32>) -> Self {
        self.position += translation;
        self
    }
    pub fn set_roll(mut self, roll: f32) -> Self {
        self.roll = roll;
        self
    }
    pub fn set_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }
    pub fn set_yaw(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self
    }
    pub fn yaw(&mut self) -> &mut f32 {
        &mut self.yaw
    }
    pub fn pitch(&mut self) -> &mut f32 {
        &mut self.pitch
    }
}

impl Camera for FPSCamera {
    fn get_camera_info(&self) -> CameraInfo {
        CameraInfo {
            fov: self.fov,
            aspect_ratio: self.aspect_ratio,
        }
    }
    fn get_origin(&self) -> Vector3<f32> {
        self.position
    }
    fn get_projection_mat(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.fov, self.aspect_ratio, self.near_clip, self.far_clip)
    }
    fn get_view_mat(&self) -> Matrix4<f32> {
        let rotation = Matrix4::look_at_rh(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(self.yaw.sin(), self.pitch.sin(), self.yaw.cos()),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let translation = Matrix4::new_translation(&(-1.0 * self.position));
        rotation * translation
    }
    fn get_mat(&self, transform: &Transform) -> Matrix4<f32> {
        let perspective_mat =
            Matrix4::new_perspective(self.fov, self.aspect_ratio, self.near_clip, self.far_clip);
        let rotation = Matrix4::look_at_rh(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(self.yaw.sin(), self.pitch.sin(), self.yaw.cos()),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let translation = Matrix4::new_translation(&(-1.0 * self.position));
        perspective_mat * rotation * translation * transform.mat()
    }
    fn move_x(&mut self, delta: f32) {
        self.position.x += delta
    }
    fn move_z(&mut self, delta: f32) {
        self.position.z += delta
    }
    fn rotate_x(&mut self, delta: f32) {
        self.yaw += delta;
    }
    fn rotate_y(&mut self, delta: f32) {
        self.pitch += delta;
    }
    fn update_zoom(&mut self, _delta: f32) {}
    fn cast_ray(&self) -> Ray {
        let rotation = Matrix4::look_at_rh(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(self.yaw.sin(), self.pitch.sin(), self.yaw.cos()),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let vec_out = rotation * Vector4::new(0.0, 0.0, -1.0, 0.0);
        let direction = Vector3::new(vec_out.x, vec_out.y, vec_out.z).normalize();
        let origin = self.position.into();
        Ray { direction, origin }
    }
}
pub struct ThirdPersonCamera {
    center: Point3<f32>,
    radius: f32,
    theta: f32,
    phi: f32,
    fov: f32,
    aspect_ratio: f32,
    near_clip: f32,
    far_clip: f32,
}
impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            center: Point3::new(50.0, 0.0, 20.0),
            radius: 100.0,
            theta: f32::consts::PI / 4.0,
            phi: 0.0,
            fov: f32::consts::PI / 4.0,
            aspect_ratio: 1.0,
            near_clip: 0.1,
            far_clip: 500.0,
        }
    }
}
impl Camera for ThirdPersonCamera {
    fn get_camera_info(&self) -> CameraInfo {
        CameraInfo {
            fov: self.fov,
            aspect_ratio: self.aspect_ratio,
        }
    }
    fn get_origin(&self) -> Vector3<f32> {
        Vector3::new(
            self.radius * self.theta.sin() * self.phi.sin(),
            self.radius * self.theta.cos(),
            self.radius * self.theta.sin() * self.phi.cos(),
        )
    }
    fn get_projection_mat(&self) -> Matrix4<f32> {
        Matrix4::new_perspective(self.fov, self.aspect_ratio, self.near_clip, self.far_clip)
    }
    fn get_view_mat(&self) -> Matrix4<f32> {
        let position = Point3::new(
            self.radius * self.theta.sin() * self.phi.sin(),
            self.radius * self.theta.cos(),
            self.radius * self.theta.sin() * self.phi.cos(),
        );
        let rotation: Matrix4<f32> = Matrix4::look_at_rh(
            &position,
            &Vector3::new(0.0, 0.0, 0.0).into(),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let translation = Matrix4::new_translation(
            &(-1.0 * Vector3::new(self.center.x, self.center.y, self.center.z)),
        );
        rotation * translation
    }

    fn move_x(&mut self, delta: f32) {
        self.center.x += delta
    }

    fn move_z(&mut self, delta: f32) {
        self.center.z += delta
    }
    fn rotate_x(&mut self, delta: f32) {
        self.phi += delta;
    }

    fn rotate_y(&mut self, delta: f32) {
        self.theta += delta;
    }
    fn update_zoom(&mut self, delta: f32) {
        self.radius += delta * self.radius
    }
    fn cast_ray(&self) -> Ray {
        let camera_position = self.radius
            * Vector3::new(
                self.theta.sin() * self.phi.sin(),
                self.theta.cos(),
                self.theta.sin() * self.phi.cos(),
            );
        let origin = self.center + camera_position;
        let direction = (-1.0 * camera_position).normalize();
        Ray { direction, origin }
    }
}
#[system]
pub fn insert_debug_ray(
    command_buffer: &mut CommandBuffer,
    #[resource] graphics: &mut Context,
    #[resource] model_manager: &mut AssetManager<sukakpak::Mesh>,
) {
    let texture = graphics
        .build_texture(&RgbaImage::from_pixel(
            100,
            100,
            Rgba::from([20, 200, 200, 200]),
        ))
        .expect("failed to build texture");
    let model = model_manager.insert(
        graphics
            .build_mesh(
                sukakpak::MeshAsset::new_cube(),
                DrawableTexture::Texture(&texture),
            )
            .expect("failed to build mesh"),
    );

    command_buffer.push((RayDebug {}, vec![(model, Transform::default())]));
}
pub struct RayDebug {}

#[system(for_each)]
pub fn debug_ray(
    ray_debug: &RayDebug,
    data: &mut Vec<(AssetHandle<sukakpak::Mesh>, Transform)>,
    #[resource] camera: &mut Box<dyn Camera>,
) {
    let ray = camera.cast_ray();
    let mesh = data[0].0.clone();
    data.clear();
    println!("ray: {}", ray);

    let ray = parry3d::query::Ray {
        dir: ray.direction,
        origin: ray.origin,
    };
    *data = (0..50)
        .map(|i| {
            let pos = ray.point_at(i as f32 * 1.0);
            let pos_v = Vector3::new(pos.x, pos.y, pos.z);
            let scale = i as f32 * 0.1;
            (
                mesh.clone(),
                Transform::default()
                    .translate(pos_v)
                    .set_scale(scale * Vector3::new(1.0, 1.0, 1.0)),
            )
        })
        .collect();
}
