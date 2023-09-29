use std::io::Read;

use ash::{ self, vk };

use shaderc::ShaderKind;

use crate::{log_err, vk_check, check_err};

/// ### fn is_extension( ... ) -> bool
/// *Compares a files extension to the string provided.*
/// <pre>
/// - Params
///     file_path:      &std::path::Path    <i>// file_name to check</i>
///     file_ext:       &str                <i>// file_ext to check against</i>
/// - Return
///     bool    <i>True if the provided file_ext string matches that on the file_name</i>
/// </pre>
fn is_extension(file_path: &std::path::Path, file_ext: &str) -> bool
{
    if file_path.extension().unwrap().to_str().unwrap() == file_ext 
    {
        return true;
    } else {
        return false;
    }
}

/// ### fn read_file_to_string( ... ) -> String
/// *Reads and returns the contents of a file to type String.*
/// <pre>
/// - Params
///     file_path:      &std::path::Path
/// - Return
///     String
/// </pre>
fn read_file_to_string(file_path: &std::path::Path) -> String
{
    let mut file = check_err!( std::fs::File::open(file_path) )
        .expect(format!("Unable to open file {}", file_path.to_str().unwrap()).as_str());
    
    let mut code = String::new();
    check_err!( file.read_to_string(&mut code) )
        .expect(format!("Unable to read file {}", file_path.to_str().unwrap()).as_str());

    code
}

/// ### fn get_shaderc_shaderkind_from_filename( ... ) -> shaderc::ShaderKind
/// *Get the shaderc::ShaderKind from file name.*
/// <pre>
/// - Param
///     file_path:      &std::path::Path
/// - Return
///     shaderc::ShaderKind
/// </pre>
pub fn get_shaderc_shaderkind_from_filename(file_path: &std::path::Path) -> ShaderKind
{
    if is_extension(&file_path, "vert") { return ShaderKind::Vertex; }
    if is_extension(&file_path, "frag") { return ShaderKind::Fragment; }
    if is_extension(&file_path, "comp") { return ShaderKind::Compute; }
    if is_extension(&file_path, "geom") { return ShaderKind::Geometry; }
    if is_extension(&file_path, "tesc") { return ShaderKind::TessControl; }
    if is_extension(&file_path, "tese") { return ShaderKind::TessEvaluation; }

    log_err!("Shader file extension for file {} is not supported.\nPlease be sure the following extensions are used: \n\t'.vert' '.frag' '.comp' '.geom' '.tesc' '.tese'", file_path.to_str().unwrap());

    panic!("Shader file extension not supported!");
}

/// ### fn get_vk_shader_stage_from_shaderc_shaderkind( ... ) -> vk::ShaderStageFlags
/// *Get the vk::ShaderStageFlags from shaderc::ShaderKind*
/// <pre>
/// - Param
///     shader_kind:     shaderc::ShaderKind
/// - Return
///     vk::ShaderStageFlags
/// </pre>
pub fn get_vk_shader_stage_from_shaderc_shaderkind(shader_kind: ShaderKind) -> vk::ShaderStageFlags
{
    match shader_kind
    {
        ShaderKind::Vertex => { return vk::ShaderStageFlags::VERTEX; },
        ShaderKind::Fragment => { return vk::ShaderStageFlags::FRAGMENT; },
        ShaderKind::Compute => { return vk::ShaderStageFlags::COMPUTE; },
        ShaderKind::Geometry => { return vk::ShaderStageFlags::GEOMETRY; },
        ShaderKind::TessControl => { return vk::ShaderStageFlags::TESSELLATION_CONTROL; },
        ShaderKind::TessEvaluation => { return vk::ShaderStageFlags::TESSELLATION_EVALUATION; }
        _ => 
        {
            panic!("Shader stage not supported not supported!");
        }   
    }
}

/// ### fn get_vk_shader_stage_from_filename( ... ) -> vk::ShaderStageFlags
/// *Get vk::ShaderStageFlags from file_name*
/// <pre>
/// - Params
///     file_path:      &std::path::Path
/// - Return
///     vk::ShaderStageFlags
/// </pre>
pub fn get_vk_shader_stage_from_filename(file_path: &std::path::Path) -> vk::ShaderStageFlags
{
    get_vk_shader_stage_from_shaderc_shaderkind(get_shaderc_shaderkind_from_filename(file_path))
}

/// ### fn compile_shader_to_spirv( ... ) -> Vec\<u32\>
/// *Compiles a shader source file into a binary in format Vec\<u32\>*
/// <pre>
/// - Params
///     file_path:      &std::path::Path
/// - Return
///     Vec&lt;u32&gt;      // Spirv binary
/// </pre>
pub fn compile_shader_to_spirv(file_path: &std::path::Path) -> Vec<u32>
{
    let source = read_file_to_string(file_path);
    let shader_kind = get_shaderc_shaderkind_from_filename(file_path);

    let compiler = shaderc::Compiler::new().unwrap();

    let compilation = check_err!( 
        compiler.compile_into_spirv(
            &source.as_str(), 
            shader_kind, 
            file_path.to_str().unwrap(),
            "main", 
            None
        )
    ).unwrap();

    compilation.as_binary().to_vec()
}

/// ### fn create_vk_shader_modue( ... ) -> vk::ShaderModule
/// <pre>
/// - Params
///     device:     &ash::Device
///     spirv:      &Vec&lt;u32&gt;
/// - Return
///     vk::ShaderModule
/// </pre>
pub fn create_vk_shader_module(device: &ash::Device, spirv: &Vec<u32>) -> vk::ShaderModule
{
    let create_info = vk::ShaderModuleCreateInfo
    {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        code_size: spirv.len(),
        p_code: spirv.as_ptr(),
        ..Default::default()
    };

    unsafe { vk_check!(device.create_shader_module(&create_info, None)).unwrap() }
}

/// ### SpVkShaderModule
/// <pre>
/// - Members
///     handle:     vk::ShaderModule,
///     spirv:      Vec&lt;u32&gt;
/// </pre>
pub struct SpVkShaderModule
{
    pub handle:     vk::ShaderModule,
    pub spirv:      Vec<u32>,
    pub stage:      vk::ShaderStageFlags
}

impl SpVkShaderModule
{
    /// ### fn SpVkShaderModule::new(...) -> SpVkShaderModule
    /// *Creates an instance of SpVkShaderModule*
    /// <pre>
    /// - Params
    ///     device:         &ash::Device
    ///     file_path:      &std::path::Path
    /// - Return
    ///     SpVkShaderModule
    /// </pre>
    pub fn new(device: &ash::Device, file_path: &std::path::Path) -> Self
    {
        let spirv = compile_shader_to_spirv(file_path);
        let handle = create_vk_shader_module(device, &spirv);
        let stage = get_vk_shader_stage_from_filename(file_path);

        Self{ handle, spirv, stage }
    }

    /// ### fn SpVkShaderModule::destroy(&mut self, ...)
    /// *Destroys the Vulkan resources contained in an instance of SpVkShaderModule*
    /// <pre>
    /// - Param
    ///     <b>&mut self</b>
    ///     device:     &ash::Device
    /// </pre>
    pub fn destroy(&mut self, device: &ash::Device)
    {
        self.spirv.clear();
        unsafe
        {
            device.destroy_shader_module(self.handle, None);
        }
    }

    /// ### fn get_vk_pipeline_info_shader_stage (&self) -> vk::PipelineShaderStageCreateInfo
    /// *Returns a populated vk::PipelineShaderStageCreateInfo struct for convenience*
    /// <pre>
    /// - Params
    ///     <b>&self</b>
    /// - Return
    ///     vk::ShaderStageCreateInfo
    /// </pre>
    pub fn get_vk_pipeline_info_shader_stage(&self) -> vk::PipelineShaderStageCreateInfo
    {
        vk::PipelineShaderStageCreateInfo
        {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: self.stage,
            module: self.handle,
            p_name: "main".as_ptr() as *const i8,
            p_specialization_info: std::ptr::null()
        }
    }
}


