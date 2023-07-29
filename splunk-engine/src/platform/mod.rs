#![allow(unused_variables)]

pub mod event_types;

use winit::{
    event::{ DeviceEvent, DeviceId, WindowEvent, Event },
    event_loop::{ EventLoop, ControlFlow }, window::WindowId, 
};

use crate::core::application::Application;


pub fn main_loop(mut app: Application, evloop: EventLoop<()>)
{
    evloop.run(move | events, _, control_flow|
    {
        *control_flow = ControlFlow::Poll;
        match events
        {
            Event::WindowEvent { window_id, event } =>
            {
                handle_window_events(&mut app, event, window_id, control_flow);
            },
            Event::DeviceEvent { device_id, event } =>
            {
                handle_device_events(&mut app, event, device_id, control_flow);
            },
            Event::MainEventsCleared => {},
            Event::RedrawRequested(_window_id) => 
            {

            },
            Event::RedrawEventsCleared => {},
            Event::LoopDestroyed => {},
            Event::Suspended => {},
            Event::Resumed => {}
            _ => {}
        }
    });
}

pub fn handle_window_events(app: &mut Application, events: WindowEvent, _window_id: WindowId, control_flow: &mut ControlFlow)
{

}

pub fn handle_device_events(app: &mut Application, events: DeviceEvent, device_id: DeviceId, control_flow: &mut ControlFlow)
{

}