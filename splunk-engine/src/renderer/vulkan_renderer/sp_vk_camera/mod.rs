// use ash::{self, vk};

use nalgebra_glm as glm;

#[derive(Debug, Copy, Clone)]
pub struct CamView
{
    pub pos:        glm::Vec3,
    pub front:      glm::Vec3,
    pub up:         glm::Vec3,
}

impl CamView
{
    pub fn get_matrix(&self) -> glm::Mat4
    {
        // glm::look_at(&self.pos, &(self.pos + self.front), &self.up)
        glm::look_at(&self.pos, &self.front, &self.up)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CamProjection
{
    pub aspect:     f32,
    pub fov:        f32,
    pub near:       f32,
    pub far:        f32,
}

pub const Y_CORRECTION: glm::Mat4 = glm::Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
);

impl CamProjection
{
    pub fn get_matrix(&self) -> glm::Mat4
    {
        let mut proj = glm::perspective(self.aspect, self.fov, self.near, self.far);
        proj.m22 *= -1.0;
        proj
    }
}

pub struct SpCamera
{
    pub view:           CamView,
    pub projection:     CamProjection,
    // proj_dirty:         bool,
}

#[repr(C)]
pub struct SpCameraUniformData
{
    // pub model:   [f32; 16],
    pub view:    [f32; 16],
    pub proj:    [f32; 16]
}

impl SpCameraUniformData
{
    pub fn new() -> Self
    {
        Self
        {
            // model: glm::Mat4::identity().as_slice()[..].try_into().unwrap(),
            view: glm::Mat4::identity().as_slice()[..].try_into().unwrap(),
            proj: glm::Mat4::identity().as_slice()[..].try_into().unwrap() 
        }
    }

    pub fn update(&mut self, camera: &SpCamera)
    {
        // self.model = model.as_slice()[..].try_into().unwrap();
        self.view = camera.view.get_matrix().as_slice()[..].try_into().unwrap();
        self.proj = camera.projection.get_matrix().as_slice()[..].try_into().unwrap();
    }
}