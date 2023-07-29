use ash::{
    vk::{self, DebugUtilsMessengerEXT},
    Instance, Entry,
    extensions::{
        khr::Surface,
        ext::DebugUtils,
    },
};

use winit::window::Window;
use raw_window_handle::{ HasRawDisplayHandle, HasRawWindowHandle };

use crate::{ vk_check, vk_validate_info, vk_validate_warn, vk_validate_err, log_err, log_info };

use std::ffi::{ CString, CStr };
use std::os::raw::c_void;

#[cfg(debug_assertions)]
pub const VALIDATION: bool = true;
#[cfg(not(debug_assertions))]
pub const VALIDATION: bool = false;

pub struct AshVkDebugLayers
{
    pub utils: DebugUtils,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

impl AshVkDebugLayers
{
        /// Creates debug utils for validation layer callbacks
    pub fn new(entry: &Entry, instance: &Instance) -> Self
    {
        let utils = ash::extensions::ext::DebugUtils::new(entry, instance);

        let messenger_ci = vk::DebugUtilsMessengerCreateInfoEXT
        {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            p_next: std::ptr::null(),
            flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
            message_severity:
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type:
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_callback),
            p_user_data: std::ptr::null_mut(),
        };

        let messenger = unsafe 
        {  
            vk_check!( utils.create_debug_utils_messenger(&messenger_ci, None) ).unwrap() 
        };
        
        Self
        {
            utils,
            messenger
        }
    }

    pub fn destroy(&self)
    {
        unsafe { self.utils.destroy_debug_utils_messenger(self.messenger, None); }
    }

}

pub struct AshVkSurface
{
    pub loader: Surface,
    pub handle: vk::SurfaceKHR
}

impl AshVkSurface
{
    pub fn new(window: &Window, entry: &Entry, instance: &Instance) -> Self
    {
        let loader = Surface::new(&entry, &instance);

        let handle  = unsafe{  vk_check!(ash_window::create_surface(&entry, &instance, window.raw_display_handle(), window.raw_window_handle(), None)).unwrap() };
    
        Self{ loader, handle }
    }

    pub fn destroy(&self)
    {
        unsafe { self.loader.destroy_surface(self.handle, None); }
    }
}

pub struct VulkanLoader
{
    pub entry: Entry,
    pub instance: Instance,
    pub debug_layer: Option<AshVkDebugLayers>,
    pub surface: AshVkSurface
}

impl VulkanLoader
{
    /// Creates a VulkanLoader Struct
    pub fn new(window: &Window, app_name: CString, app_version: u32) -> Self
    {
        let entry = unsafe { Entry::load().map_err(|e| { log_err!(e); } ).unwrap() };

        let engine_name = CString::new("Splunk Engine").unwrap();
        let engine_version = vk::make_api_version(0, 0, 1, 0);
        let instance = create_vk_instance(window, &entry, app_name, app_version, engine_name, engine_version);

        let mut debug_layer: Option<AshVkDebugLayers> = None;
        if VALIDATION
        {
            debug_layer = Some(AshVkDebugLayers::new(&entry, &instance));
        }

        let surface = AshVkSurface::new(&window, &entry, &instance);

        Self
        {
            entry,
            instance,
            debug_layer,
            surface
        }
    }

    pub fn destroy(&self)
    {
        self.surface.destroy();
        if VALIDATION
        {
            self.debug_layer.as_ref().unwrap().destroy();
        }
        unsafe { self.instance.destroy_instance(None); }
    }
}


/// Creates a VkInstance handle
pub fn create_vk_instance(window: &Window, entry: &Entry, app_name: CString, app_version: u32, engine_name: CString, engine_version: u32) -> Instance
{
    log_info!("Creating VkInstance handle...");

    let app_info = vk::ApplicationInfo
    {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: std::ptr::null(),
        p_application_name: app_name.as_ptr(),
        application_version: app_version,
        p_engine_name: engine_name.as_ptr(),
        engine_version: engine_version,
        api_version: ash::vk::API_VERSION_1_3
    };

    let extension_names = vk_check!( ash_window::enumerate_required_extensions(window.raw_display_handle()) ).unwrap();
    let debug_utils_name = &[DebugUtils::name().as_ptr()];
    let extension_names = [ extension_names,  debug_utils_name ].concat();

    let mut layers: Vec<*const i8> = vec![];
    if VALIDATION
    {
        layers.push("VK_LAYER_KHRONOS_validation".as_ptr() as *const i8);
    }

    let create_info = vk::InstanceCreateInfo
    {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        enabled_extension_count: extension_names.len() as u32,
        pp_enabled_extension_names: extension_names.as_slice().as_ptr(),
        enabled_layer_count: layers.len() as u32,
        pp_enabled_layer_names: layers.as_slice().as_ptr(),
    };
    
    let instance = unsafe { vk_check!( entry.create_instance(&create_info, None)).unwrap() };
    
    log_info!("VkInstance handle created.");

    return instance;
}


unsafe extern "system" fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut c_void
    ) -> vk::Bool32
{
    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
    {
        return vk::FALSE;
    }
    let message = CStr::from_ptr((*p_callback_data).p_message);
    let types = match message_type
    {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]"
    };
    match message_severity
    {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO =>
        {
            let severity = "[Info]";
            vk_validate_info!("Validation:", severity, types, message.to_str().unwrap());
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING =>
        {
            let severity = "[Warning]";
            vk_validate_warn!("Validation:", severity, types, message.to_str().unwrap());
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => 
        {
            let severity = "[Error]";
            vk_validate_err!("Validation:", severity, types, message.to_str().unwrap());
        }
        _ => {}
    };
    return vk::FALSE;
}


/// Creates debug utils for validation layer callbacks
pub fn create_debug_callback(entry: &Entry, instance: &Instance) -> (DebugUtils, DebugUtilsMessengerEXT)
{
    let debut_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

    if !VALIDATION
    {
        return  (debut_utils_loader, ash::vk::DebugUtilsMessengerEXT::null());
    }

    let messenger_ci = vk::DebugUtilsMessengerCreateInfoEXT
    {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: std::ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity:
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type:
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_callback),
        p_user_data: std::ptr::null_mut(),
    };

    let utils_messenger = unsafe 
    {  
        vk_check!( debut_utils_loader.create_debug_utils_messenger(&messenger_ci, None) ).unwrap() 
    };
    
    (debut_utils_loader, utils_messenger)
}
