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

/// ### SpVkDebugLayers
/// *Used for validation layer messages*
/// <pre>
/// - Members
///     utils:          ext::DebugUtils             <i>// Creates and destroys messenger</i>
///     messenger:      vk::DebugUtilsMessengerEXT  <i>// Responsible for validation layer messages</i>
/// </pre>
pub struct SpVkDebugLayers
{
    pub utils: DebugUtils,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

impl SpVkDebugLayers
{
    /// ### fn SpVkDebugLayers::new( ... ) -> SpVkDebugLayers
    /// *Creates an instance of SpVkDebugLayers*
    /// <pre>
    /// - Params
    ///     entry:      &ash::Entry
    ///     instance:   &ash::Instance
    /// - Return
    ///     SpVkDebuLayers
    /// </pre>
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

    /// ### fn SpVkDebugLayers::destroy(&self)
    /// *Destroys an instance of SpVkDebugLayers*
    /// <pre>
    /// - Param
    ///     <b>&self</b>
    /// </pre>
    pub fn destroy(&self)
    {
        unsafe { self.utils.destroy_debug_utils_messenger(self.messenger, None); }
    }

}

/// ### SpVkSurface struct
/// *The surface is responsible for the drawing on the screen*
/// *VkSurface convenience struct*
/// <pre>
/// - Members
///     loader:     khr::Surface        <i>// Creates and destroys surface handle</i>
///     handle:     vk::SurfaceKHR      <i>// VkSurfaceKHR handle</i>
/// </pre>
pub struct SpVkSurface
{
    pub loader:     Surface,
    pub handle:     vk::SurfaceKHR
}

impl SpVkSurface
{
    /// ### fn SpVkSurface::new( ... ) -> SpVkSurface
    /// *Creates an instance of SpVkSurface*
    /// <pre>
    /// - Params
    ///     window:         &winit::window::Window
    ///     entry:          &ash::Entry
    ///     instance:       &ash::Instance
    /// - Return
    ///     SpVkSurface
    /// </pre>
    pub fn new(window: &Window, entry: &Entry, instance: &Instance) -> Self
    {
        let loader = Surface::new(&entry, &instance);

        let handle  = unsafe{  vk_check!(ash_window::create_surface(&entry, &instance, window.raw_display_handle(), window.raw_window_handle(), None)).unwrap() };
    
        Self{ loader, handle }
    }

    /// ### fn SpVkSurface::destroy(&self)
    /// *Destroys instance of SpVkSurface*
    /// <pre>
    /// - Param
    ///     <b>&self</b>
    /// </pre>
    pub fn destroy(&self)
    {
        unsafe { self.loader.destroy_surface(self.handle, None); }
    }
}

/// ### SpVkLoader struct
/// *Contains handles necessary to load and debug vulkan*
/// <pre>
/// - Members
///     entry:          &ash::Entry
///     instance:       &ash::Instance
///     debug_layer:    Option&lt;SpVkDebugLayers&gt;
///     surface:        SpVkSurface
/// </pre>
pub struct SpVkLoader
{
    pub entry:          Entry,
    pub instance:       Instance,
    pub debug_layer:    Option<SpVkDebugLayers>,
    pub surface:        SpVkSurface
}

impl SpVkLoader
{
    
    /// ### fn SpVkLoader::new( ... ) -> SpVkLoader
    /// *Creates an instance SpVkLoader struct. Loads vulkan and debuggers.*
    /// <pre>
    /// - Param
    ///     window:         &winit::window::Window
    ///     app_name:       CString
    ///     app_version:    u32
    /// - Return
    ///     SpVkLoader
    /// </pre>
    pub fn new(window: &Window, app_name: CString, app_version: u32) -> Self
    {
        let entry = unsafe { Entry::load().map_err(|e| { log_err!(e); } ).unwrap() };

        let engine_name = CString::new("Splunk Engine").unwrap();
        let engine_version = vk::make_api_version(0, 0, 1, 0);
        let instance = create_vk_instance(window, &entry, app_name, app_version, engine_name, engine_version);

        let mut debug_layer: Option<SpVkDebugLayers> = None;
        if VALIDATION
        {
            debug_layer = Some(SpVkDebugLayers::new(&entry, &instance));
        }

        let surface = SpVkSurface::new(&window, &entry, &instance);

        Self
        {
            entry,
            instance,
            debug_layer,
            surface
        }
    }

    /// ### SpVkLoader::destroy(&self)
    /// *Destroys an instance of SpVkLoader, destroying vulkan loaders, surface, and debuggers.*
    /// <pre>
    /// - Param
    ///     <b>&self</b>
    /// </pre>
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


/// ### fn create_vk_instance( ... ) -> ash::Instance
/// *Creates a VkInstance handle*
/// <pre>
/// - Params
///     window:             &winit::window::Window
///     entry:              &ash::Entry
///     app_name:           CString
///     app_version:        u32
///     engine_name:        CString
///     engine_version:     u32
/// - Return
///     ash::Instance
/// </pre>
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

    let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    let mut layers: Vec<*const i8> = vec![];
    if VALIDATION
    {
        layers.push(validation_layer.as_ptr());
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
        pp_enabled_layer_names: layers.as_ptr(),
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


/// ### fn create_debug_callback( ... ) -> (ext::DebugUtils, vk::definitions::DebugUtilsMessengerEXT)
/// *Creates debug utils for validation layer callbacks*
/// <pre>
/// - Param
///     entry: &ash::Entry
///     instance: &ash::Instance
/// - Return
///     (ash::extensions::ext::DebugUtils, ash::vk::definitions::DebugUtilsMessengerEXT)
/// </pre>
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
