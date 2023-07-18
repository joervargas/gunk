
pub trait RendererLayer
{
    fn update(&mut self);

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>);
    
    fn render(
        &mut self, encoder: &mut wgpu::CommandEncoder, 
        surface_view: &wgpu::TextureView, depth_texture_view: Option<&wgpu::TextureView>, 
        camera_bind_group: &wgpu::BindGroup,
        light_bind_group: &wgpu::BindGroup,
    ) -> Result<(), wgpu::SurfaceError>;
}