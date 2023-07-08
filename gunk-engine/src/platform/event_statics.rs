// use std::sync::Mutex;

use super::event_types::{KeyEvent, MouseMotionEvent, MouseButtonEvent};


// pub static mut A_KEY : Mutex<KeyEvent> = Mutex::new(KeyEvent::new(None));
// pub static mut D_KEY : Mutex<KeyEvent> = Mutex::new(KeyEvent::new(None));
// pub static mut S_KEY : Mutex<KeyEvent> = Mutex::new(KeyEvent::new(None));
// pub static mut W_KEY : Mutex<KeyEvent> = Mutex::new(KeyEvent::new(None));


// pub static mut MouseMotion : Mutex<MouseMotionEvent> = Mutex::new(MouseMotionEvent::new(None));

// pub static mut LEFT_CLICK : Mutex<MouseButtonEvent> = Mutex::new(MouseButtonEvent::new(None));
// pub static mut RIGHT_CLICK : Mutex<MouseButtonEvent> = Mutex::new(MouseButtonEvent::new(None));
// pub static mut MIDDLE_CLICK : Mutex<MouseButtonEvent> = Mutex::new(MouseButtonEvent::new(None));

pub static mut A_KEY : KeyEvent = KeyEvent::new(None);
pub static mut D_KEY : KeyEvent = KeyEvent::new(None);
pub static mut S_KEY : KeyEvent = KeyEvent::new(None);
pub static mut W_KEY : KeyEvent = KeyEvent::new(None);


pub static mut MOUSE_MOTION : MouseMotionEvent = MouseMotionEvent::new(None);

pub static mut LEFT_CLICK : MouseButtonEvent = MouseButtonEvent::new(None);
pub static mut RIGHT_CLICK : MouseButtonEvent = MouseButtonEvent::new(None);
pub static mut MIDDLE_CLICK : MouseButtonEvent = MouseButtonEvent::new(None);