use std::io::Read;

use ash::{ self, vk };

use shaderc::ShaderKind;

use crate::{ log_err, vk_check };


fn is_extension(file_name: &std::path::Path, file_ext: &str) -> bool
{
    if file_name.extension().unwrap().to_str().unwrap() == file_ext 
    {
        return true;
    } else {
        return false;
    }
}

fn read_shader_file(file_name: &std::path::Path) -> String
{
    let mut file = std::fs::File::open(file_name).map_err(|e| {log_err!(e);} )
        .expect(format!("Unable to open file {}", file_name.to_str().unwrap()).as_str());

    let mut code = String::new();
    file.read_to_string(&mut code).map_err(|e| { log_err!(e); })
        .expect(format!("Unable to read file {}", file_name.to_str().unwrap()).as_str());

    code
}

pub fn get_shaderc_shaderkind_from_filename(file_name: &std::path::Path) -> ShaderKind
{
    if is_extension(&file_name, "vert") { return ShaderKind::Vertex; }
    if is_extension(&file_name, "frag") { return ShaderKind::Fragment; }
    if is_extension(&file_name, "comp") { return ShaderKind::Compute; }
    if is_extension(&file_name, "geom") { return ShaderKind::Geometry; }
    if is_extension(&file_name, "tesc") { return ShaderKind::TessControl; }
    if is_extension(&file_name, "tese") { return ShaderKind::TessEvaluation; }

    log_err!("Shader file extension for file {} is not supported.\nPlease be sure the following extensions are used: \n\t'.vert' '.frag' '.comp' '.geom' '.tesc' '.tese'", file_name.to_str().unwrap());

    panic!("Shader file extension not supported!");
}

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

pub fn get_vk_shader_stage_from_filename(file_name: &std::path::Path) -> vk::ShaderStageFlags
{
    get_vk_shader_stage_from_shaderc_shaderkind(get_shaderc_shaderkind_from_filename(file_name))
}

pub fn create_vk_shader_module(device: &ash::Device, spirv: &Vec<u32>) -> vk::ShaderModule
{
    let create_info = vk::ShaderModuleCreateInfo
    {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: spirv.len() * std::mem::size_of::<u32>(),
        p_code: spirv.as_ptr()
    };

    unsafe {
        vk_check!( device.create_shader_module(&create_info, None) ).unwrap()
    }
}

pub fn destroy_vk_shader_module(device: &ash::Device, shader_module: vk::ShaderModule)
{
    unsafe {
        device.destroy_shader_module(shader_module, None);
    }
}

pub struct SpVkShaderModule
{
    pub handle: vk::ShaderModule,
    pub spirv: Vec<u32>
}

pub fn sp_create_shader_module(device: &ash::Device, file_name: &std::path::Path) -> SpVkShaderModule
{
    let code = read_shader_file(file_name);
    let shader_kind = get_shaderc_shaderkind_from_filename(file_name);

    let compiler = shaderc::Compiler::new().unwrap();
    // let mut compile_options = shaderc::CompileOptions::new().unwrap();
    // compile_options.add_macro_definition("EP", Some("main"));
    let binary_result = compiler.compile_into_spirv(
        code.as_str(), 
        shader_kind, 
        file_name.to_str().unwrap() , 
        
        "main", 
        None
    ).map_err(|e| { log_err!(e); } ).unwrap();

    let spirv = binary_result.as_binary().to_vec();
    let handle = create_vk_shader_module(device, &spirv);

    SpVkShaderModule
    {
        handle,
        spirv
    }
}

pub fn sp_destroy_shader_module(device: &ash::Device, shader_module: &mut SpVkShaderModule)
{
    shader_module.spirv.clear();
    destroy_vk_shader_module(device, shader_module.handle);
}

pub fn sp_get_vk_shader_create_info(shader_module: &SpVkShaderModule, stage: vk::ShaderStageFlags, entry_point: &str) -> vk::PipelineShaderStageCreateInfo
{
    vk::PipelineShaderStageCreateInfo
    {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::PipelineShaderStageCreateFlags::empty(),
        stage,
        module: shader_module.handle,
        p_name: entry_point.as_ptr() as *const i8,
        p_specialization_info: std::ptr::null()
    }
}