
use winit::event::{ElementState, MouseButton};

use super::event_types::MouseMotionData;
use super::event_statics::*;

pub fn handle_mouse_motion(motion_delta: &(f64, f64))
{
    let motion_data: MouseMotionData = MouseMotionData{ x: motion_delta.0, y: motion_delta.1 };

    unsafe { MOUSE_MOTION.notify_listener(&motion_data); }
}

pub fn handle_mouse_button(button: &MouseButton, state: &ElementState)
{
    match button 
    {
        MouseButton::Left =>
        {
            unsafe{ LEFT_CLICK.notify_listener(state); }
        },
        MouseButton::Right =>
        {
            unsafe{ RIGHT_CLICK.notify_listener(state); }
        }
        MouseButton::Middle =>
        {
            unsafe{ MIDDLE_CLICK.notify_listener(state); }
        }
        _ => {}
    }
}