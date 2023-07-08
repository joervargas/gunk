#![allow(dead_code)]

use std::string::String;

use winit::{
    event_loop::{EventLoop}, 
    window::{Window, WindowBuilder}, 
    dpi::LogicalSize};

use crate::platform::{main_loop};
use crate::renderer;

// use crate::platform::{self, run_loop};

pub struct AppConfig
{
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub web_id: String,
}

pub struct Application
{
    pub config: AppConfig,
    pub window: Window,
    pub renderer: renderer::WgpuContext,
}

impl Application
{
    pub fn new(config: AppConfig) -> (Self, EventLoop<()>)
    {   
        let evloop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(config.title.clone())
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .build(&evloop).expect("Failed to build the Window!");

        #[cfg(target_arch="wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            window.set_inner_size(PhysicalSize::new(config.width, config.height));
            
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id(config.web_id.clone().as_str())?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }

        let renderer = renderer::WgpuContext::new(&window);

        let app = Self
        { 
            config,
            window,
            renderer, 
        };

        (app, evloop)
    }
    
    pub fn init(&mut self)
    {
        cfg_if::cfg_if!
        {
            if #[cfg(target_arch="wasm32")]
            {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            }
            else
            {
                env_logger::init();
            }
        }
    }

    pub fn run(self, evloop: EventLoop<()>)
    {
        main_loop(self, evloop);
    }

}

impl Drop for Application
{
    fn drop(&mut self) 
    {
        
    }
}