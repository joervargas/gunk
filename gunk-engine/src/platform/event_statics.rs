// use std::sync::Mutex;

use super::event_types::{KeyEvent, MouseMotionEvent, MouseButtonEvent};

// Keys
pub static mut A_KEY : KeyEvent = KeyEvent::new(None);
pub static mut D_KEY : KeyEvent = KeyEvent::new(None);
pub static mut S_KEY : KeyEvent = KeyEvent::new(None);
pub static mut W_KEY : KeyEvent = KeyEvent::new(None);

// Mouse motion
pub static mut MOUSE_MOTION : MouseMotionEvent = MouseMotionEvent::new(None);

// Mouse buttons
pub static mut LEFT_CLICK : MouseButtonEvent = MouseButtonEvent::new(None);
pub static mut RIGHT_CLICK : MouseButtonEvent = MouseButtonEvent::new(None);
pub static mut MIDDLE_CLICK : MouseButtonEvent = MouseButtonEvent::new(None);