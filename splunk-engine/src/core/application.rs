

use winit::{
    event_loop::EventLoop,
    window::{ Window, Fullscreen }, 
    dpi::PhysicalSize,
};

use std::{string::String, ffi::CString};

use crate::platform::main_loop;
use crate::renderer::{
    renderer_utils::GfxRenderer,
    vulkan_renderer::vulkan_renderer::VulkanRenderer,
};

/// ### AppConfig struct
/// *Configurations for Application and Window startup*
/// <pre>
/// - Members
///     title:              String
///     width:              u32
///     height:             u32
///     b_fullscreen:       bool    <i>// is fullscreen?
///     b_resizeable:       bool    <i>// is resizable?
///     b_border:           bool    <i>// has border?
/// </pre>
pub struct AppConfig
{
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub b_fullscreen: bool, // is fullscreen
    pub b_resizable: bool, // is resizable
    pub b_border: bool,  // has border
}

/// ### Application struct
/// *Contains members necessary for a functioning application*
pub struct Application
{
    pub config: AppConfig,
    pub window: Window,
    pub renderer: Box<dyn GfxRenderer>
}

impl Application
{
    /// ### Application::new( ... ) -> (Application, EventLoop\<()\>)
    /// *Creates an instance of the application.<br> Also return an EventLoop.*
    /// <pre>
    /// - Params
    ///     config:     AppConfig
    /// - Return
    ///     (Application, EventLoop&lt;()&gt;)
    /// </pre>
    pub fn new(config: AppConfig) -> (Self, EventLoop<()>)
    {
        let evloop: EventLoop<()> = EventLoop::new();
        let window: Window = Window::new(&evloop).unwrap();

        if config.b_resizable
        {
            window.set_resizable(true);
        }

        if config.b_fullscreen
        {
            let video_mode = window.current_monitor().unwrap().video_modes().next().unwrap();
            let fullscreen = Fullscreen::Exclusive(video_mode);
            window.set_fullscreen(Some(fullscreen));
        } else {
            window.set_inner_size(PhysicalSize::new(config.width, config.height));
        }
        
        let renderer = Box::new(
            VulkanRenderer::new(
                &window, 
                CString::new(config.title.clone()).unwrap(), 
                ash::vk::make_api_version(0, 0, 1, 0)
            )
        );
        let app = Self{
            config,
            window,
            renderer
        };

        (app, evloop)
    }
    

    pub fn init(&mut self)
    {
        self.renderer.init();
    }

    /// #### fn Application::run(self, ... )
    /// *Application loop*
    /// <pre>
    /// <b><i>Note:</i></b> <i>Consumes self and evloop.</i>
    /// - Params
    ///     <b><i>self</i></b>
    ///     evloop:     EventLoop&lt;()&gt;
    /// </pre>
    pub fn run(self, evloop: EventLoop<()>)
    {
        main_loop(self, evloop);
    }

}


impl Drop for Application
{
    fn drop(&mut self)
    {
        self.renderer.destroy();
    }
}