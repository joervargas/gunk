pub mod wgpu_utils;
pub mod renderer_layers;
pub mod texture;
pub mod camera;
pub mod model;
pub mod light;

// use std::sync::{RwLock, Arc};

use nalgebra_glm as glm;

use winit::window;

use self::{
    wgpu_utils::*, 
    renderer_layers::{
        layer::RendererLayer,
        // model_layer::ModelLayer, 
        batch3d_layer::Batch3DLayer, model_layer::ModelLayer,
    }, 
    camera::CameraObject,
};

// use crate::platform::{ event_statics::*, event_types::* };

pub struct WgpuContext
{
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter: wgpu::Adapter,
    pub surface_info: WgpuSurfaceInfo,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: texture::Texture,
    
    pub camera_obj: CameraObject,
    pub scene_light: light::SceneLight,

    pub layers: Vec<Box::<dyn RendererLayer>>,
}

impl WgpuContext
{

    pub fn new(window: &window::Window) -> Self
    {
        let size = window.inner_size();

        let instance = create_wgpu_instance();

        let surface = unsafe{ instance.create_surface(&window) }.unwrap();

        let adapter = create_wgpu_adapter(&instance, &surface);

        let (device, queue) = request_wgpu_device_and_queue(&adapter);

        let surface_info = set_wgpu_surface_info(&surface, &adapter, size.width, size.height);
        surface.configure(&device, &surface_info.configuration);

        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_info.configuration, "depth_texture");

        let camera = camera::Camera
        {
            pos: glm::Vec3::new(0.0, 1.0, 2.0), // eye position
            front: glm::Vec3::new(0.0, 0.0, 0.0),
            up: glm::Vec3::new(0.0, 1.0, 0.0).into(),
            aspect: size.width as f32 / size.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        let camera_obj = CameraObject::new(&device, camera);

        let scene_light = light::SceneLight::new(&device, "scene light", [2.0, 2.0, 2.0], [1.0, 1.0, 1.0]);

        let mut layers: Vec<Box<dyn RendererLayer>> = Vec::new();
        // order is important
        // first is group 0, second is group 1, ...
        let scene_bind_group_layouts = [&camera_obj.bind_group_layout, &scene_light.bind_group_layout];
        layers.push(Box::new(ModelLayer::new(&device, &queue, &surface_info, &scene_bind_group_layouts)));
        layers.push(Box::new(Batch3DLayer::new(&device, &queue, &surface_info, &scene_bind_group_layouts)));

        Self{
            surface,
            device,
            queue,
            adapter,
            surface_info,
            size,
            
            depth_texture,
            
            camera_obj,
            scene_light,

            layers,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        if new_size.width > 0 && new_size.height > 0
        {
            self.size = new_size;
            self.surface_info.configuration.width = new_size.width;
            self.surface_info.configuration.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_info.configuration);

            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.surface_info.configuration, "depth texture");
        }
    }

    pub fn update(&mut self)
    {
        let cam_control = self.camera_obj.controller.read().unwrap();
        cam_control.update_camera(&mut self.camera_obj.camera);
        drop(cam_control);

        self.camera_obj.uniform.update(&self.camera_obj.camera.get_view_matrix(), &self.camera_obj.camera.get_projection());
        self.queue.write_buffer(&self.camera_obj.buffer, 0, bytemuck::cast_slice(&[self.camera_obj.uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let cmd_encoder_desc = &wgpu::CommandEncoderDescriptor{ label: Some("Render Encoder"), };
        let mut encoder = self.device.create_command_encoder(&cmd_encoder_desc);

        for layer in self.layers.iter_mut()
        {
            layer.render(& mut encoder, &view, Some(&self.depth_texture.view), &self.camera_obj.bind_group, &self.scene_light.bind_group)?;
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


}

impl Drop for WgpuContext
{
    fn drop(&mut self)
    {

    }
}