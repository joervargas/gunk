use std::io::{BufReader, Cursor};

// use cfg_if::cfg_if;
// use wgpu::util::DeviceExt;

use crate::renderer::{ model, texture, wgpu_utils };

// #[cfg(target_arch = "wasm32")]
// fn format_url(file_name: &str) -> reqwest::Url
// {
//     let window = websys::window().unwrap();
//     let location = window.location();
//     let mut origin = location.origin().unwrap();
//     if !origin.ends_with("gunky-editor")
//     {
//         origin = format!("{}/gunky-editor", orgin);
//     }
//     let base = reqwest::Url::parse(&format!("{}/", origin)).unwrap();
//     base.join(file_name).unwrap()
// }

// pub async fn load_string(file_name: &str) -> anyhow::Result<String>
// {
//     cfg_if!
//     {
//         if #[cfg(target_arch = "wasm32")]
//         {
//             let url = format_url(file_name);
//             let txt = reqwest::get(url)
//                 .await?
//                 .text()
//                 .await?;
//         } else {
//             let  path = std::path::Path::new(env!("OUT_DIR"))
//                 .join("res")
//                 .join(file_name);
//             let txt = std::fs::read_to_string(path)?;
//         }
//     }

//     Ok(txt)
// }

pub fn load_string(file_name: &str) -> anyhow::Result<String>
{
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;

    // let out_dir = std::env::var("OUT_DIR")?;
    // let path = std::path::Path::new(out_dir.as_str())
    //     .join("res")
    //     .join(file_name);
    // let txt = std::fs::read_to_string(path)?;

    Ok(txt)
}

// pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>>
// {
//     cfg_if!
//     {
//         if #[cfg(target_arch = "wasm32")]
//         {
//             let url = format_url(file_name);
//             let data = reqwest::get(url)
//                 .await?
//                 .bytes()
//                 .await?
//                 .to_vec();
//         } else {
//             let path = std::path::Path::new(env!("OUT_DIR"))
//                 .join("res")
//                 .join(file_name);
//             let data = std::fs::read(path)?;
//         }
//     }

//     Ok(data)
// }

pub fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>>
{
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;

    Ok(data)
}

// pub async fn load_texture( 
//         file_name: &str, 
//         device: &wgpu::Device, 
//         queue: &wgpu::Queue
//     ) -> anyhow::Result<texture::Texture>
// {
//     let data = load_binary(file_name).await?;
//     texture::Texture::from_bytes(device, queue, &data, Some(file_name))
// }

pub fn load_texture(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> anyhow::Result<texture::Texture>
{
    let data = load_binary(file_name)?;
    texture::Texture::from_bytes(&device, &queue, &data, Some(file_name))
}

// pub async fn load_model(
//         file_name: &str,
//         device: &wgpu::Device,
//         queue: &wgpu::Queue,
//         layout: &wgpu::BindGroupLayout,
//     ) -> anyhow::Result<model::Model>
// {
//     let obj_text = load_string(file_name).await?;
//     let obj_cursor = Cursor::new(obj_text);
//     let mut obj_reader = BufReader::new(obj_cursor);

//     let load_options = tobj::LoadOptions
//     {
//         triangulate: true,
//         single_index: true,
//         ..Default::default()
//     };
//     let ( models, obj_materials ) = tobj::load_obj_buf_async(
//         &mut obj_reader, &load_options, 
//         |p| async 
//         {
//            let mat_text = load_string(&p).await.unwrap();
//            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text))) 
//         }).await?;

//     let mut materials = Vec::new();
//     for m in obj_materials?
//     {
//         let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;
//         let bind_group_desc = wgpu::BindGroupDescriptor
//         {
//             layout,
//             entries: &[
//                 wgpu::BindGroupEntry
//                 {
//                     binding: 0,
//                     resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
//                 },
//                 wgpu::BindGroupEntry
//                 {
//                     binding: 1,
//                     resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
//                 }
//             ],
//             label: None
//         };
//         let bind_group = device.create_bind_group(&bind_group_desc);

//         materials.push(model::Material{ name: m.name, diffuse_texture, bind_group})
//     }

//     let meshes = models.into_iter().map(
//         |m|
//         {
//             let vertices = (0..m.mesh.positions.len() / 3).map(
//                 |i|
//                 {
//                     model::ModelVertex
//                     {
//                         position: [
//                             m.mesh.positions[i * 3],
//                             m.mesh.positions[i * 3 + 1],
//                             m.mesh.positions[i * 3 + 2]
//                         ],
//                         tex_coords: [
//                             m.mesh.texcoords[i * 2], 
//                             m.mesh.texcoords[i * 2 + 1]
//                         ],
//                         normal: [
//                             m.mesh.normals[i * 3],
//                             m.mesh.normals[i * 3 + 1],
//                             m.mesh.normals[i * 3 + 2]
//                         ]
//                     }
//                 }
//             ).collect::<Vec<_>>();

//             let vertex_buffer = wgpu_utils::create_wgpu_buffer::<model::ModelVertex>(device, format!("{:?} Vertex Buffer", file_name).as_str(), wgpu::BufferUsages::VERTEX, &vertices);
//             let index_buffer = wgpu_utils::create_wgpu_buffer::<u32>(device, format!("{:?} Index Buffer", file_name).as_str(), wgpu::BufferUsages::INDEX, &m.mesh.indices);

//             model::Mesh
//             {
//                 name: file_name.to_string(),
//                 vertex_buffer,
//                 index_buffer,
//                 num_elements: m.mesh.indices.len() as u32,
//                 material: m.mesh.material_id.unwrap_or(0)
//             }
//         }
//     ).collect::<Vec<_>>();

//     Ok(model::Model { meshes, materials })
// }

pub fn load_model(
        file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> anyhow::Result<model::Model>
{
    let obj_text = load_string(file_name)?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let load_options = tobj::LoadOptions
    {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    let ( models, obj_materials ) = tobj::load_obj_buf(
        &mut obj_reader, &load_options, 
        |p|
        {
            let mat_text = load_string(&p.to_str().unwrap()).unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text))) 
        })?;

    let mut materials = Vec::new();
    for m in obj_materials?
    {
        let diffuse_texture = load_texture(&m.diffuse_texture.as_ref().unwrap().as_str(), &device, &queue)?;
        dbg!();

        let bind_group_desc = wgpu::BindGroupDescriptor
        {
            layout,
            entries: &[
                wgpu::BindGroupEntry
                {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry
                {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                }
            ],
            label: None
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        materials.push(model::Material{ name: m.name, diffuse_texture, bind_group})
    }

    let meshes = models.into_iter().map(
        |m|
        {
            let vertices = (0..m.mesh.positions.len() / 3).map(
                |i|
                {
                    model::ModelVertex
                    {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2]
                        ],
                        tex_coords: [
                            m.mesh.texcoords[i * 2], 
                            m.mesh.texcoords[i * 2 + 1]
                        ],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2]
                        ]
                    }
                }
            ).collect::<Vec<_>>();

            let vertex_buffer = wgpu_utils::create_wgpu_buffer::<model::ModelVertex>(device, format!("{:?} Vertex Buffer", file_name).as_str(), wgpu::BufferUsages::VERTEX, &vertices);
            let index_buffer = wgpu_utils::create_wgpu_buffer::<u32>(device, format!("{:?} Index Buffer", file_name).as_str(), wgpu::BufferUsages::INDEX, &m.mesh.indices);

            model::Mesh
            {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0)
            }
        }
    ).collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}