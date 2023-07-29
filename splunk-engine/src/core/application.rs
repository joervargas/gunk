

use winit::{
    event_loop::EventLoop,
    window::{ Window, Fullscreen }, 
    dpi::PhysicalSize,
};

use std::string::String;

use crate::platform::main_loop;

pub struct AppConfig
{
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub is_fullscreen: bool,
    pub is_resizable: bool,
    pub has_border: bool,
}

pub struct Application
{
    pub config: AppConfig,
    pub window: Window,
}

impl Application
{
    pub fn new(config: AppConfig) -> (Self, EventLoop<()>)
    {
        let evloop: EventLoop<()> = EventLoop::new();
        let window: Window = Window::new(&evloop).unwrap();

        if config.is_resizable
        {
            window.set_resizable(true);
        }

        if config.is_fullscreen
        {
            let video_mode = window.current_monitor().unwrap().video_modes().next().unwrap();
            let fullscreen = Fullscreen::Exclusive(video_mode);
            window.set_fullscreen(Some(fullscreen));
        } else {
            window.set_inner_size(PhysicalSize::new(config.width, config.height));
        }
        
        let app = Self{
            config,
            window
        };

        (app, evloop)
    }
    
    pub fn init(&mut self)
    {

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