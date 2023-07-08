#![allow(unused_variables)]

pub mod event_types;
pub mod event_statics;
mod handle_key_events;
mod handle_mouse_events;

use crate::core::application::Application;

use winit::{
    event::{ Event, WindowEvent, DeviceEvent, DeviceId }, 
    window::WindowId, 
    event_loop::{ControlFlow, EventLoop}
};

pub fn main_loop(mut app: Application, evloop: EventLoop<()>)
    {
        evloop.run(move |events, _, control_flow|
        {
            *control_flow = ControlFlow::Poll;
            match events
            {
                Event::WindowEvent { window_id, event } =>
                {
                    handle_window_events(&mut app, event, window_id, control_flow)
                },
                Event::DeviceEvent { device_id, event } =>
                {
                    handle_device_events(&mut app, event, device_id, control_flow)
                },
                Event::MainEventsCleared => { app.window.request_redraw(); },
                Event::RedrawRequested(_window_id) =>
                {
                    app.renderer.update();
                    match app.renderer.render()
                    {
                        Ok(_) => {},
                        Err(wgpu::SurfaceError::Lost) => { app.renderer.resize(app.renderer.size) },
                        Err(wgpu::SurfaceError::OutOfMemory) => { control_flow.set_exit(); },
                        Err(e) => { eprintln!("{:?}", e) } 
                    }
                }
                Event::RedrawEventsCleared =>
                {

                }
                Event::LoopDestroyed =>
                {

                },
                Event::Suspended =>
                {

                },
                Event::Resumed =>
                {

                }
                _ => {}
            }
        });
    }

pub fn handle_window_events(app: &mut Application, events: WindowEvent, _window_id: WindowId, control_flow: &mut ControlFlow)
{
    match events
    {
        WindowEvent::CloseRequested =>
        {
            control_flow.set_exit();
        },
        WindowEvent::KeyboardInput { device_id, input, is_synthetic } =>
        {
            handle_key_events::hande_key_events(&input);
        },
        WindowEvent::CursorMoved { device_id, position, .. } =>
        {

        },
        WindowEvent::MouseInput { device_id, state, button, .. } =>
        {
            handle_mouse_events::handle_mouse_button(&button, &state)
        },
        WindowEvent::MouseWheel { device_id, delta, phase, .. } =>
        {

        },
        WindowEvent::AxisMotion { device_id, axis, value } =>
        {

        },
        WindowEvent::Resized(physical_size) =>
        {
            app.renderer.resize(physical_size);
        },
        WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size } =>
        {
            app.renderer.resize(*new_inner_size);
        }
        WindowEvent::Focused(b_is_focused) =>
        {

        },
        _ => {}
    }
}

pub fn handle_device_events(app: &mut Application, events: DeviceEvent, device_id: DeviceId, control_flow: &mut ControlFlow)
{
    match events
    {
        DeviceEvent::Key(input) =>
        {

        },
        DeviceEvent::MouseMotion { delta } =>
        {
            handle_mouse_events::handle_mouse_motion(&delta);
        },
        DeviceEvent::Button { button, state } =>
        {

        },
        DeviceEvent::MouseWheel { delta } =>
        {

        },
        DeviceEvent::Motion { axis, value } =>
        {

        },
        _ => {}
    }
}