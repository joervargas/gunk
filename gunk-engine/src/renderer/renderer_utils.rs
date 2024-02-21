
use winit::window::Window;

pub trait GfxRenderer
{
    fn init(&self);

    fn destroy(&mut self);

    fn update(&mut self, window: &Window, delta_time: f32);

    fn render(&mut self, window: &Window, delta_time: f32);

    fn resized(&mut self);

    fn wait_idle(&self);
}

pub fn to_asset_path(file_name: &str) -> std::path::PathBuf
{
    std::path::Path::new(env!("OUT_DIR"))
        .join("assets")
        .join(file_name)
}

pub fn to_shader_path(file_name: &str) -> std::path::PathBuf
{
    std::path::Path::new(env!("OUT_DIR"))
        .join("shaders")
        .join(file_name)
}

