#![allow(unused_variables)]

pub mod event_types;

use winit::{
    event::{ DeviceEvent, DeviceId, WindowEvent, Event },
    event_loop::{ EventLoop, ControlFlow, EventLoopWindowTarget }, window::WindowId, 
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

    evloop.set_control_flow(ControlFlow::Poll);
    evloop.run(move | events, elwt|
    {
        match events
        {
            Event::WindowEvent { window_id, event } =>
            {
                handle_window_events(&mut app, event, window_id, &elwt, &mut tick_counter);
            },
            Event::DeviceEvent { device_id, event } =>
            {
                handle_device_events(&mut app, event, device_id, &elwt);
            },
            // Event::MainEventsCleared => 
            // {
            //     app.window.request_redraw(); 
            // },
            // Event::RedrawRequested(_window_id) => 
            // {
            //     if !app.minimized
            //     {
            //         app.renderer.render(&app.window, delta_time);
            //         tick_counter.tick_frame();
            //     }
            // },
            // Event::RedrawEventsCleared => {},
            // Event::LoopDestroyed => 
            // { 
            //     app.renderer.wait_idle();
            // },
            Event::Suspended => {},
            Event::Resumed => {}
            _ => {}
        }

        app.window.request_redraw();
    }).expect("Something in Eventloop went wrong");
}

/// ### fn handle_window_events( ... )
/// *Window events go here*
pub fn handle_window_events(app: &mut Application, events: WindowEvent, _window_id: WindowId, elwt: &EventLoopWindowTarget<()>, tick_counter: &mut FPSLimiter)
{
    match events
    {
        WindowEvent::CloseRequested =>
        {
            elwt.exit();
        }
        WindowEvent::Resized(size) => 
        {
            if size.width == 0 || size.height == 0
            {
                app.minimized = true;
            }
            app.resized();
        },
        WindowEvent::RedrawRequested => 
        {
            if !app.minimized
            {
                let delta_time = tick_counter.delta_time();
                app.renderer.render(&app.window, delta_time);
                tick_counter.tick_frame();
            }
        },
        _ => {}
    }
}

/// ### fn handle_device_events( ... )
/// *Device events go here*
pub fn handle_device_events(app: &mut Application, events: DeviceEvent, device_id: DeviceId, elwt: &EventLoopWindowTarget<()>)
{

}