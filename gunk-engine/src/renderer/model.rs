// use tobj::{Mesh, Material};

use super::texture;
use std::ops::Range;

pub trait Vertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex
{
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        use std::mem;
        wgpu::VertexBufferLayout { 
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute
                {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute
                {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute
                {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct Model
{
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>
}

// impl Model
// {
//     pub fn new(
//             file_name: &str,
//             device: &wgpu::Device, 
//             queue: &wgpu::Queue,
//             layout: &wgpu::BindGroupLayout
//         ) -> anyhow::Result<Self>
//     {
//         let mut reader = BufReader::new(File::open(file_name)?);
//         let load_options = tobj::LoadOptions
//         {
//             triangulate: true,
//             single_index: true,
//             ..Default::default()
//         };

//         let (models, obj_materials) = tobj::load_obj_buf(
//             &mut reader, 
//             &load_options,
//             |p| // p = path of material
//             {
//                 // let mat_path = std::fs::read_to_string(p)?;
//                 // tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_path)))
//                 let mat_file = File::open(p).expect(format!("Failed to open material file {}", p.to_str().unwrap()).as_str());
//                 tobj::load_mtl_buf(&mut BufReader::new(mat_file)) 
//             }
//         ).expect(format!("Failed to read {}", file_name).as_str());

//         let obj_materials = obj_materials.expect(format!("Failed to load MTL file for {}", file_name).as_str());

//         let mut materials = Vec::new();
//         for m in obj_materials
//         {
//             let texture_path = m.diffuse_texture.as_ref().unwrap().as_str();
//             let diffuse_bytes = std::fs::read(texture_path).expect(format!("Failed to read file {}", texture_path).as_str());
//             let diffuse_texture = texture::Texture::from_bytes(&device, &queue, &diffuse_bytes, Some(texture_path)).unwrap();
//             // let diffuse_texture = texture::Texture::
//             // let diffuse_texture = load_texture(&m.diffuse_texture.as_ref().unwrap().as_str(), &device, &queue)?;
//             dbg!();

//             let bind_group_desc = wgpu::BindGroupDescriptor
//             {
//                 layout,
//                 entries: &[
//                     wgpu::BindGroupEntry
//                     {
//                         binding: 0,
//                         resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
//                     },
//                     wgpu::BindGroupEntry
//                     {
//                         binding: 1,
//                         resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
//                     }
//                 ],
//                 label: None
//             };
//             let bind_group = device.create_bind_group(&bind_group_desc);

//             materials.push(Material{ name: m.name, diffuse_texture, bind_group})
//         }

//         let meshes = models.into_iter().map(
//             |m|
//             {
//                 let vertices = (0..m.mesh.positions.len() / 3).map(
//                     |i|
//                     {
//                         ModelVertex
//                         {
//                             position: [
//                                 m.mesh.positions[i * 3],
//                                 m.mesh.positions[i * 3 + 1],
//                                 m.mesh.positions[i * 3 + 2]
//                             ],
//                             tex_coords: [
//                                 m.mesh.texcoords[i * 2], 
//                                 m.mesh.texcoords[i * 2 + 1]
//                             ],
//                             normal: [
//                                 m.mesh.normals[i * 3],
//                                 m.mesh.normals[i * 3 + 1],
//                                 m.mesh.normals[i * 3 + 2]
//                             ]
//                         }
//                     }
//                 ).collect::<Vec<_>>();
    
//                 let vertex_buffer = wgpu_utils::create_wgpu_buffer::<ModelVertex>(device, format!("{:?} Vertex Buffer", file_name).as_str(), wgpu::BufferUsages::VERTEX, &vertices);
//                 let index_buffer = wgpu_utils::create_wgpu_buffer::<u32>(device, format!("{:?} Index Buffer", file_name).as_str(), wgpu::BufferUsages::INDEX, &m.mesh.indices);
    
//                 Mesh
//                 {
//                     name: file_name.to_string(),
//                     vertex_buffer,
//                     index_buffer,
//                     num_elements: m.mesh.indices.len() as u32,
//                     material: m.mesh.material_id.unwrap_or(0)
//                 }
//             }
//         ).collect::<Vec<_>>();

//         Ok(Self
//         {
//             meshes,
//             materials
//         })
//     }
// }

pub struct Mesh
{
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize
}

pub struct Material
{
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup
}

pub trait DrawModel<'a>
{
    fn draw_mesh(&mut self, mesh: &'a Mesh);
    fn draw_mesh_instanced(&mut self, mesh: &'a Mesh, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a
{
    fn draw_mesh(&mut self, mesh: &'b Mesh)
    {
        self.draw_mesh_instanced(mesh, 0..1);
    }

    fn draw_mesh_instanced(&mut self, mesh: &'b Mesh, instances: Range<u32>)
    {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}