#![allow(unused_variables)]

pub mod event_types;

use winit::{
    event::{ DeviceEvent, DeviceId, WindowEvent, Event },
    event_loop::{ EventLoop, ControlFlow }, window::WindowId, 
};

use crate::core::{ 
    application::Application, 
    fps_limiter::FPSLimiter
};

/// ### fn main_loop( ... )
/// *The main device loop*
/// <pre>
/// <b><i>Note:</i></b <i>fn main_loop() consumes Application and EventLoop<()></i>
/// - Params
///     app:        mut Application
///     evloop:     EventLoop<()>
/// </pre>
pub fn main_loop(mut app: Application, evloop: EventLoop<()>)
{
    let mut tick_counter = FPSLimiter::new();

    evloop.run(move | events, _, control_flow|
    {
        let delta_time = tick_counter.delta_time();
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
            Event::MainEventsCleared => 
            {
                app.window.request_redraw(); 
            },
            Event::RedrawRequested(_window_id) => 
            {
                if !app.minimized
                {
                    app.renderer.render(&app.window, delta_time);
                    tick_counter.tick_frame();
                }
            },
            Event::RedrawEventsCleared => {},
            Event::LoopDestroyed => 
            { 
                app.renderer.wait_idle();
            },
            Event::Suspended => {},
            Event::Resumed => {}
            _ => {}
        }
    });
}

/// ### fn handle_window_events( ... )
/// *Window events go here*
pub fn handle_window_events(app: &mut Application, events: WindowEvent, _window_id: WindowId, control_flow: &mut ControlFlow)
{
    match events
    {
        WindowEvent::CloseRequested =>
        {
            *control_flow = ControlFlow::Exit;
        }
        WindowEvent::Resized(size) => 
        {
            if size.width == 0 || size.height == 0
            {
                app.minimized = true;
            }
            app.resized();
        },
        _ => {}
    }
}

/// ### fn handle_device_events( ... )
/// *Device events go here*
pub fn handle_device_events(app: &mut Application, events: DeviceEvent, device_id: DeviceId, control_flow: &mut ControlFlow)
{

}