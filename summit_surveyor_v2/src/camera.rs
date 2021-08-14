use std::f32;
use sukakpak::nalgebra::{Matrix4, Point3, Vector3};
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
#[derive(Clone)]
pub struct Camera {
    position: Vector3<f32>,
    pitch: f32,
    yaw: f32,
    roll: f32,
    fov: f32,
    aspect_ratio: f32,
    near_clip: f32,
    far_clip: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            fov: f32::consts::PI / 2.0,
            aspect_ratio: 1.0,
            near_clip: 0.1,
            far_clip: 100.0,
        }
    }
}
impl Camera {
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
    pub fn to_vec(&self, transform: &Transform) -> Vec<u8> {
        let perspective_mat =
            Matrix4::new_perspective(self.fov, self.aspect_ratio, self.near_clip, self.far_clip);
        let rotation = Matrix4::face_towards(
            &Point3::new(0.0, 0.0, 0.0),
            &Point3::new(self.yaw.sin(), self.pitch.sin(), self.yaw.cos()),
            &Vector3::new(0.0, 1.0, 0.0),
        );
        let translation = Matrix4::new_translation(&(-1.0 * self.position));
        (perspective_mat * rotation * translation * transform.mat())
            .as_slice()
            .iter()
            .map(|f| f.to_ne_bytes())
            .flatten()
            .collect()
    }
}
