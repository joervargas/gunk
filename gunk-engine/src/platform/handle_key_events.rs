use winit::event::{KeyboardInput, VirtualKeyCode};

use super::event_statics::*;


pub fn hande_key_events(input: &KeyboardInput)
{
    if input.virtual_keycode.is_none() { return; }

    let key_code = input.virtual_keycode.unwrap();
    match key_code
    {
        VirtualKeyCode::A => 
        { 
            unsafe{ A_KEY.notify_listener(&input.state); }
        },
        VirtualKeyCode::D => 
        {
            unsafe{ D_KEY.notify_listener(&input.state); }
        },
        VirtualKeyCode::S => 
        {
            unsafe{ S_KEY.notify_listener(&input.state); }
        },
        VirtualKeyCode::W => 
        {
            unsafe{ W_KEY.notify_listener(&input.state); }
        },
        _ => {}
    }
}