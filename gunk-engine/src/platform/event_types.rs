#![allow(dead_code)]

use std::{sync::{ Arc, RwLock }, ops::DerefMut};

use winit::event::ElementState;


// Key Events

pub trait IKeyListener
{
    fn fire_callback(&self, state: &ElementState);
}

pub struct DelegateKeyListener<T>
{
    obj: Arc<RwLock<T>>,
    callback: fn(&mut T, state: &ElementState),
}

impl<T> DelegateKeyListener<T>
{
    pub fn new(obj: Arc<RwLock<T>>, callback: fn(&mut T, state: &ElementState)) -> Self
    {
        Self{ obj, callback }
    }
}

impl<T> IKeyListener for DelegateKeyListener<T>
{
    fn fire_callback(&self, state: &ElementState)
    {
        let mut w_obj = self.obj.write().unwrap();
        (self.callback)(w_obj.deref_mut(), state);
        drop(w_obj);
    }
}

pub struct KeyEvent
{
    listener: Option<Box<dyn IKeyListener>>
}

impl KeyEvent
{
    pub const fn new(listener: Option<Box<dyn IKeyListener>>) -> Self
    {
        Self{ listener }
    }

    pub fn set_listener(&mut self, new_listener: Option<Box<dyn IKeyListener>>)
    {
        self.listener = new_listener;
    }

    pub fn notify_listener(&self, state: &ElementState)
    {
        if self.listener.is_some()
        {
            self.listener.as_ref().unwrap().fire_callback(state);
        }
    }
}


// Mouse Events
// Mouse Motion Events

pub struct MouseMotionData
{
    pub x: f64,
    pub y: f64,
}

pub trait IMouseMotionListener
{
    fn fire_callback(&self, mouse_motion_data: &MouseMotionData);
}

pub struct DelegateMouseMotionListener<T>
{
    obj: Arc<RwLock<T>>,
    callback: fn(&mut T, mouse_motion_data: &MouseMotionData),
}

pub struct MouseMotionEvent
{
    listener: Option<Box<dyn IMouseMotionListener>>
}

impl MouseMotionEvent
{
    pub const fn new(listener: Option<Box<dyn IMouseMotionListener>>) -> Self
    {
        Self{ listener }
    }

    pub fn set_listener(&mut self, new_listener: Option<Box<dyn IMouseMotionListener>>)
    {
        self.listener = new_listener;
    }

    pub fn notify_listener(&self, mouse_motion_data: &MouseMotionData)
    {
        if self.listener.is_some()
        {
            self.listener.as_ref().unwrap().fire_callback(mouse_motion_data);
        }
    }
}

// Mouse Button Events

// pub struct MouseButtonData
// {
//     pub button: u32,
//     pub state: ElementState,
// }

pub trait IMouseButtonListener
{
    fn fire_callback(&self, state: &ElementState);
}

pub struct DelegateMouseButtonListener<T>
{
    obj: Arc<RwLock<T>>,
    callback: fn(&mut T, state: &ElementState),
}

pub struct MouseButtonEvent
{
    listener: Option<Box<dyn IMouseButtonListener>>
}

impl MouseButtonEvent
{
    pub const fn new(listener: Option<Box<dyn IMouseButtonListener>>) -> Self
    {
        Self{ listener }
    }

    pub fn set_listener(&mut self, new_listener: Option<Box<dyn IMouseButtonListener>>)
    {
        self.listener = new_listener;
    }

    pub fn notify_listener(&self, state: &ElementState)
    {
        if self.listener.is_some()
        {
            self.listener.as_ref().unwrap().fire_callback(&state);
        }
    }
}