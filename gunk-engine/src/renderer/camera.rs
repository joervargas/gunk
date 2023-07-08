// #![allow(dead_code)]

use std::sync::{ RwLock, Arc };
use nalgebra_glm as glm;
use bytemuck;
use winit::event::ElementState;

use super::wgpu_utils;

use crate::platform::{ event_types::*, event_statics::* };

pub const DEPTH_CORRECTION_MATRIX: glm::Mat4 = glm::Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

// const YAW: f32 = -90.0;
// const PITCH: f32 = 0.0;
// const SPEED: f32 = 2.5;
// const SENSITIVITY: f32 = 0.1;
// const ZOOM: f32 = 45.0;

pub struct Camera
{
    pub pos: glm::Vec3, // eye position
    pub front: glm::Vec3,
    // pub right: glm::Vec3,
    pub up: glm::Vec3,
    // pub world_up: glm::Vec3,
    // pub yaw: f32,
    // pub pitch: f32,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera
{
    // pub fn new(pos: glm::Vec3, aspect: f32, fov: f32, near: f32, far: f32) -> Self
    // {
    //     let front = Vec3::new(
    //         yaw.to_radians().cos() * pitch.to_radians().cos(),
    //         pitch.to_radians().sin(),
    //         self.yaw.to_radians().sin() * pitch.to_radians().cos(),
    //     );
    //     front = front.normalize();
    //     right = glm::cross(&front, &world_up);
    //     up = glm::cross(&self.right, &front);


    // }

    pub fn get_view_matrix(&self) -> glm::Mat4
    {
        glm::look_at(&self.pos, &self.front, &self.up)
    }

    pub fn get_projection(&self) -> glm::Mat4
    {
        DEPTH_CORRECTION_MATRIX * glm::perspective(self.aspect, self.fov, self.near, self.far)
    }

    pub fn get_bind_group_layout() -> wgpu::BindGroupLayoutDescriptor<'static>
    {
        wgpu::BindGroupLayoutDescriptor
        {
            label: Some("camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ]
        }
    }


}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct  CameraUniform
{
    pub view: [f32; 16],
    pub proj: [f32; 16],
}

impl CameraUniform
{
    pub fn new() -> Self
    {
        Self
        {
            view: glm::Mat4::identity().as_slice()[..].try_into().unwrap(),
            proj: glm::Mat4::identity().as_slice()[..].try_into().unwrap(),
        }
    }

    pub fn update(&mut self, view: &glm::Mat4, proj: &glm::Mat4)
    {
        self.view = view.as_slice()[..].try_into().unwrap();
        self.proj = proj.as_slice()[..].try_into().unwrap();
    }

}

pub struct CameraController
{
    pub speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool
}

impl CameraController
{
    pub fn new(speed: f32) -> Self
    {
        
        Self
        {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false
        }
    }

    pub fn update_camera(&self, camera: &mut Camera)
    {
        let forward = (camera.front - camera.pos).normalize();
        // let forward_normal = forward.normalize();
        // let forward_magnitude = forward.magnitude();

        let right = forward.cross(&camera.up);

        if self.is_forward_pressed
        {
            camera.pos += forward * self.speed;
        }
        if self.is_backward_pressed
        {
            camera.pos -= forward * self.speed;
        }

        if self.is_right_pressed
        {
            camera.pos +=  right * self.speed;
        }
        if self.is_left_pressed
        {
            camera.pos -= right * self.speed;
        }

    }

    pub fn move_up(&mut self, state: &ElementState)
    {
        match state 
        {
            ElementState::Pressed => { self.is_up_pressed = true; },
            ElementState::Released => { self.is_up_pressed = false; }
        }
    }

    pub fn move_down(&mut self, state: &ElementState)
    {
        match state 
        {
            ElementState::Pressed => { self.is_down_pressed = true; },
            ElementState::Released => { self.is_down_pressed = false; }
        }
    }

    pub fn move_forward(&mut self, state: &ElementState)
    {
        match state 
        {
            ElementState::Pressed => { self.is_forward_pressed = true; },
            ElementState::Released => { self.is_forward_pressed = false; }
        }
    }

    pub fn move_backward(&mut self, state: &ElementState)
    {
        match state 
        {
            ElementState::Pressed => { self.is_backward_pressed = true; },
            ElementState::Released => { self.is_backward_pressed = false; }
        }
    }

    pub fn move_left(&mut self, state: &ElementState)
    {
        match state 
        {
            ElementState::Pressed => { self.is_left_pressed = true; },
            ElementState::Released => { self.is_left_pressed = false; }
        }
    }

    pub fn move_right(&mut self, state: &ElementState)
    {
        match state 
        {
            ElementState::Pressed => { self.is_right_pressed = true; },
            ElementState::Released => { self.is_right_pressed = false; }
        }
    }
}

pub struct CameraObject
{
    pub camera: Camera,
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub controller: Arc<RwLock<CameraController>>,
}

impl CameraObject
{
    pub fn new(device: &wgpu::Device, camera: Camera) ->Self
    {
        let mut uniform = CameraUniform::new();
        uniform.update(&camera.get_view_matrix(), &camera.get_projection());

        let buffer = wgpu_utils::create_wgpu_buffer::<CameraUniform>(
            &device, 
            "Camera Buffer", 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, 
            &[uniform]
        );

        let camera_bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor
        {
            label: Some("camera bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry
                {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
            ]
        };
        let bind_group_layout = device.create_bind_group_layout(&camera_bind_group_layout_desc);
        let camera_bind_group_descriptor = wgpu::BindGroupDescriptor
        {
            label: Some("camera_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry 
                { 
                    binding: 0, 
                    resource: buffer.as_entire_binding() 
                }
            ]
        };
        let bind_group = device.create_bind_group(&camera_bind_group_descriptor);

        let controller = Arc::new(RwLock::new(CameraController::new(0.2)));

        let move_forward = DelegateKeyListener::<CameraController>::new(controller.clone(), CameraController::move_forward);
        let move_backward = DelegateKeyListener::<CameraController>::new(controller.clone(), CameraController::move_backward);
        let move_right = DelegateKeyListener::<CameraController>::new(controller.clone(), CameraController::move_right);
        let move_left = DelegateKeyListener::<CameraController>::new(controller.clone(), CameraController::move_left);
        
        unsafe
        { 
            W_KEY.set_listener(Some(Box::new(move_forward)));
            S_KEY.set_listener(Some(Box::new(move_backward)));
            A_KEY.set_listener(Some(Box::new(move_left)));
            D_KEY.set_listener(Some(Box::new(move_right)));
        }

        Self
        {
            camera,
            uniform,
            buffer,
            bind_group,
            bind_group_layout,
            controller,
        }
    }

    pub fn set_event_listeners(&self)
    {
        let move_forward = DelegateKeyListener::<CameraController>::new(self.controller.clone(), CameraController::move_forward);
        let move_backward = DelegateKeyListener::<CameraController>::new(self.controller.clone(), CameraController::move_backward);
        let move_right = DelegateKeyListener::<CameraController>::new(self.controller.clone(), CameraController::move_right);
        let move_left = DelegateKeyListener::<CameraController>::new(self.controller.clone(), CameraController::move_left);
        
        unsafe
        { 
            W_KEY.set_listener(Some(Box::new(move_forward)));
            S_KEY.set_listener(Some(Box::new(move_backward)));
            A_KEY.set_listener(Some(Box::new(move_left)));
            D_KEY.set_listener(Some(Box::new(move_right)));
        }
    }

}