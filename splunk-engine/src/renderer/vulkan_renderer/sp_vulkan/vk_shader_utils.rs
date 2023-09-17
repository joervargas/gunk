use std::io::Read;

use ash::{ self, vk };

use shaderc::ShaderKind;

use crate::log_err;

/// ### fn is_extension( ... ) -> bool
/// *Compares a files extension to the string provided.*
/// <pre>
/// - Params
///     file_name:      &std::path::Path    <i>// file_name to check</i>
///     file_ext:       &str                <i>// file_ext to check against</i>
/// - Return
///     bool    <i>True if the provided file_ext string matches that on the file_name</i>
/// </pre>
fn is_extension(file_name: &std::path::Path, file_ext: &str) -> bool
{
    if file_name.extension().unwrap().to_str().unwrap() == file_ext 
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
///     file_name:      &std::path::Path
/// - Return
///     String
/// </pre>
fn read_file_to_string(file_name: &std::path::Path) -> String
{
    let mut file = std::fs::File::open(file_name).map_err(|e| {log_err!(e);} )
        .expect(format!("Unable to open file {}", file_name.to_str().unwrap()).as_str());

    let mut code = String::new();
    file.read_to_string(&mut code).map_err(|e| { log_err!(e); })
        .expect(format!("Unable to read file {}", file_name.to_str().unwrap()).as_str());

    code
}

/// ### fn get_shaderc_shaderkind_from_filename( ... ) -> shaderc::ShaderKind
/// *Get the shaderc::ShaderKind from file name.*
/// <pre>
/// - Param
///     file_name:      &std::path::Path
/// - Return
///     shaderc::ShaderKind
/// </pre>
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
///     file_name:      &std::path::Path
/// - Return
///     vk::ShaderStageFlags
/// </pre>
pub fn get_vk_shader_stage_from_filename(file_name: &std::path::Path) -> vk::ShaderStageFlags
{
    get_vk_shader_stage_from_shaderc_shaderkind(get_shaderc_shaderkind_from_filename(file_name))
}