
use glm::{IVec2, vec3};
use image::DynamicImage;
// use image::{DynamicImage, imageops::FilterType, GenericImageView};
use nalgebra_glm as glm;

use crate::log_err;

use super::gk_bitmap::{
    GkBitMap, EBitMapType
    EBitMapData, EBitMapDataMutPtr, EBitMapDataPtr, 
};
// use crate::renderer::bitmap::BitMapScalar;


/// Credit Henry J. Warren's "Hacker's Delight"
/// From http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
fn radical_inverse_vdc(bits: u32) -> f32
{
    let mut bits = bits;
    bits = (bits << 16u32) | (bits >> 16u32);
    bits = ((bits & 0x55555555u32) << 1u32) | ((bits & 0xAAAAAAAAu32) >> 1u32);
    bits = ((bits & 0x33333333u32) << 2u32) | ((bits & 0xCCCCCCCCu32) >> 2u32);
    bits = ((bits & 0x0F0F0F0Fu32) << 4u32) | ((bits & 0xF0F0F0F0u32) >> 4u32);
    bits = ((bits & 0x00FF00FFu32) << 8u32) | ((bits & 0xFF00FF00u32) >> 8u32);

    bits as f32 * 2.3283064365386963e-10f32
}

/// Credit Holger Dammertz
/// /// From http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
pub fn hammersley_2d(index: u32, num_of_points: u32) -> glm::Vec2
{
    glm::Vec2::new(index as f32/ num_of_points as f32, radical_inverse_vdc(index))
}

pub fn face_coords_to_xyz(i: i32, j: i32, face_id: i32, face_size: i32) -> glm::Vec3
{
    let a = 2.0 * i as f32 / face_size as f32;
    let b = 2.0 * j as f32 / face_size as f32;

    if face_id == 0 { return glm::Vec3::new(-1.0, a - 1.0, b - 1.0); }
    if face_id == 1 { return glm::Vec3::new(a - 1.0, -1.0, 1.0 - b); }
    if face_id == 2 { return glm::Vec3::new(1.0, a - 1.0, 1.0 - b); }
    if face_id == 3 { return glm::Vec3::new(1.0 - a, 1.0, 1.0 - b); }
    if face_id == 4 { return glm::Vec3::new(b - 1.0, a - 1.0, 1.0); }
    if face_id == 5 { return glm::Vec3::new(1.0 - b, a - 1.0, -1.0); }

    vec3(0.0, 0.0, 0.0)
}

// pub fn convolve_diffuse<T: BitMapScalar>(img: &DynamicImage, src_w: i32, src_h: i32, dst_w: i32, dst_h: i32, num_monte_carlo_samples: i32, output: &mut glm::Vec3 )
// {
//     assert!(src_w == 2 * src_h);
//     if src_w != 2 * src_h { return; }

//     let mut tmp : Vec<glm::Vec3> = Vec::new();
//     tmp.reserve((dst_w * dst_h) as usize);

//     let new_img = img.resize(dst_w as u32, dst_h as u32, FilterType::Nearest);
    
//     let src_w = dst_w;
//     let src_h = dst_h;

//     for y in 0..dst_h
//     {
//         print!("Line {} ...\n", y);
//         let theta1 : f32 = y as f32 / (dst_h as f32) * std::f32::consts::PI;

//         for x in 0..dst_w
//         {
//             let phi1 = x as f32 / (dst_w as f32) * (std::f32::consts::PI * 2.0);
//             let v1 = glm::vec3(theta1.cos() * phi1.cos(), theta1.sin() * phi1.sin(), theta1.cos());

//             let mut color = glm::Vec3::new(0.0, 0.0, 0.0);
//             let mut weight: f32 = 0.0;

//             for i in 0..num_monte_carlo_samples
//             {
//                 let h = hammersley_2d(i as u32, num_monte_carlo_samples as u32);
//                 let x1 = (h.x * src_w as f32).floor() as i32;
//                 let y1 = (h.y * src_h as f32).floor() as i32;

//                 let theta2 : f32 = (y1 as f32) / (src_h as f32) * std::f32::consts::PI;
//                 let phi2 : f32 = (x1 as f32) / (src_w as f32) * (std::f32::consts::PI * 2.0);
//                 let v2 = glm::vec3(theta2.sin() * phi2.cos(), theta2.sin() * phi2.sin(), theta2.cos());
//                 let d : f32 = 0.0_f32.max(glm::dot(&v1, &v2));
//                 if d > 0.01
//                 {
//                     color += tmp[(y1 * src_w + x1) as usize] * d;
//                     weight += d;
//                 }
//             }
//             let pixel = new_img.get_pixel((dst_w + x) as u32, y as u32).0;
//             output.x = pixel[0] as f32;
//             output.y = pixel[1] as f32;
//             output.z = pixel[2] as f32;
//         }
//      }

//     todo!()
// }

// pub fn convolve_diffuse(img: &DynamicImage, src_w: i32, src_h: i32, dst_w: i32, dst_h: i32, num_monte_carlo_samples: i32, output: )
// {
//     assert!(src_w == 2 * src_h);
//     if src_w != 2 * src_h { return; }

//     let mut tmp : Vec<glm::Vec3> = Vec::new();
//     tmp.reserve((dst_w * dst_h) as usize);

//     let new_img = img.resize(dst_w as u32, dst_h as u32, FilterType::Nearest);
    
//     let src_w = dst_w;
//     let src_h = dst_h;

//     for y in 0..dst_h
//     {
//         print!("Line {} ...\n", y);
//         let theta1 : f32 = y as f32 / (dst_h as f32) * std::f32::consts::PI;

//         for x in 0..dst_w
//         {
//             let phi1 = x as f32 / (dst_w as f32) * (std::f32::consts::PI * 2.0);
//             let v1 = glm::vec3(theta1.cos() * phi1.cos(), theta1.sin() * phi1.sin(), theta1.cos());

//             let mut color = glm::Vec3::new(0.0, 0.0, 0.0);
//             let mut weight: f32 = 0.0;

//             for i in 0..num_monte_carlo_samples
//             {
//                 let h = hammersley_2d(i as u32, num_monte_carlo_samples as u32);
//                 let x1 = (h.x * src_w as f32).floor() as i32;
//                 let y1 = (h.y * src_h as f32).floor() as i32;

//                 let theta2 : f32 = (y1 as f32) / (src_h as f32) * std::f32::consts::PI;
//                 let phi2 : f32 = (x1 as f32) / (src_w as f32) * (std::f32::consts::PI * 2.0);
//                 let v2 = glm::vec3(theta2.sin() * phi2.cos(), theta2.sin() * phi2.sin(), theta2.cos());
//                 let d : f32 = 0.0_f32.max(glm::dot(&v1, &v2));
//                 if d > 0.01
//                 {
//                     color += tmp[(y1 * src_w + x1) as usize] * d;
//                     weight += d;
//                 }
//             }
//             let pixel = new_img.get_pixel((dst_w + x) as u32, y as u32).0;
//             let p = color / weight;
//             output.x = pixel[0];
//             output.y = pixel[1];
//             output.z = pixel[2];
//             output.w = pixel[3];
//         }
//      }

//     todo!()
// }

pub fn convert_equirectangle_to_vertical_cross(bitmap: &GkBitMap) -> GkBitMap
{
    if bitmap.bm_type != EBitMapType::Type2D { return bitmap.clone(); }

    let face_size = bitmap.width as i32 / 4;
    let w = face_size * 3;
    let h = face_size * 4;

    let d = match bitmap.data
    {
        EBitMapData::UByte(_) => {
            let mut u: Vec<u8> = Vec::new();
            // u.reserve(udata.len())
            let size = (w * h) as usize * bitmap.channels * bitmap.get_bytes_per_component();
            u.reserve(size);
            EBitMapData::UByte(u)
        },
        EBitMapData::Float(_) => {
            let mut f: Vec<f32> = Vec::new();
            // f.reserve(fdata.len());
            let size = (w * h) as usize * bitmap.channels;
            f.reserve(size);
            EBitMapData::Float(f)
        }
    };
    let mut result = GkBitMap::new(w as u32, h as u32, None, bitmap.channels, d);

    let face_offsets : Vec<glm::IVec2> = vec![
        IVec2::new(face_size, face_size * 3),
        IVec2::new(0, face_size),
        IVec2::new(face_size, face_size),
        IVec2::new(face_size * 2, face_size),
        IVec2::new(face_size, 0),
        IVec2::new(face_size, face_size *2)
    ];

    let clamp_w = bitmap.width as i32 - 1;
    let clamp_h = bitmap.height as i32 - 1;

    for face in 0..6
    {
        for i in 0..face_size
        {
            for j in 0..face_size
            {
                let p: glm::Vec3 = face_coords_to_xyz(i, j, face, face_size);
                let r = p.x.hypot(p.y);
                let theta = p.y.atan2(p.x);
                let phi = p.z.atan2(r);
                // f32 source coordinates
                let uf = (2.0 * face_size as f32 * (theta + std::f32::consts::PI) / std::f32::consts::PI) as f32;
                let vf = (2.0 * face_size as f32 * (std::f32::consts::PI / 2.0 - phi) / std::f32::consts::PI) as f32;
                // 4-samples bilinear interpolation
                let u1 = (uf.floor() as i32).clamp(0, clamp_w);
                let v1 = (vf.floor() as i32).clamp(0, clamp_h);
                let u2 = (u1 + 1).clamp(0, clamp_w);
                let v2 = (v1 + 1).clamp(0, clamp_h);
                // fractional part
                let s = uf - u1 as f32;
                let t = vf - v1 as f32;
                // fetch 4-samples
                let a: glm::Vec4 = bitmap.get_pixel(u1, v1);
                let b: glm::Vec4 = bitmap.get_pixel(u2, v1);
                let c: glm::Vec4 = bitmap.get_pixel(u1, v2);
                let d: glm::Vec4 = bitmap.get_pixel(u2, v2);
                // bilinear interpolation
                let color: glm::Vec4 = a * (1.0 - s) * (1.0 - t) + b * (s) * (1.0 - t) + c * (1.0 - s) * t + d * (s) * (t);
                result.set_pixel(i + face_offsets[face as usize].x, j + face_offsets[face as usize].y , &color);
            } // j
        } // i
    } // face

    result
}

pub fn convert_vertical_cross_to_cubemap_faces(bitmap: &GkBitMap) -> GkBitMap
{
    let face_width = bitmap.width / 3;
    let face_height = bitmap.height / 4;

    let d = match bitmap.data
    {
        EBitMapData::UByte(_) => {
            let mut u: Vec<u8> = Vec::new();
            // u.reserve(udata.len())
            let size = (face_width * face_height) as usize * bitmap.channels * bitmap.get_bytes_per_component() * 6;
            u.reserve(size);
            EBitMapData::UByte(u)
        },
        EBitMapData::Float(_) => {
            let mut f: Vec<f32> = Vec::new();
            // f.reserve(fdata.len());
            let size = (face_width * face_height) as usize * bitmap.channels * 6;
            f.reserve(size);
            EBitMapData::Float(f)
        }
    };
    let mut cubemap = GkBitMap::new(face_width, face_height, Some(6), bitmap.channels, d);

    let pixel_size = cubemap.channels * cubemap.get_bytes_per_component();
    // let src = bitmap.data.as_ptr();
    // let dst = cubemap.data.as_mut_ptr();

    let src = bitmap.get_data_ptr();
    let dst = cubemap.get_data_mut_ptr();

    let mut dst_offset: isize = 0;

	/*
			------
			| +Y |
	 ----------------
	 | -X | -Z | +X |
	 ----------------
			| -Y |
			------
			| +Z |
			------
	*/

    for face in 0..6
    {
        for j in 0..face_height
        {
            for i in 0..face_width
            {
                let mut x = 0;
                let mut y = 0;

                match face
                {
                    0 => { // + X
                        x = i;
                        y = face_height + j;
                    }
                    1 => { // -X
                        x = 2 * face_width + i;
                        y = 1 * face_height + j;
                    }
                    2 => { // + Y
                        x = 2 * face_width - (i + 1);
                        y = 1 * face_height - (j + 1);
                    }
                    3 => { // - Y
                        x = 2 * face_width - (i + 1);
                        y = 3 * face_height - (j + 1);
                    }
                    4 => { // + Z
                        x = 2 * face_width - (i + 1);
                        y = bitmap.height - (j + 1);
                    }
                    5 => { // - Z
                        x = face_width + i;
                        y = face_height + j;
                    }
                    _ => {}
                }
                let src_offset = (y * bitmap.width + x) as isize * pixel_size as isize;
                // unsafe { std::ptr::copy_nonoverlapping(src.offset(src_offset), dst.offset(dst_offset),pixel_size); }
                unsafe{
                    match src
                    {
                        EBitMapDataPtr::UBytePtr(src_ptr) => {
                            if let EBitMapDataMutPtr::UByteMutPtr(dst_mut_ptr) = dst
                            {
                                std::ptr::copy_nonoverlapping(src_ptr.offset(src_offset), dst_mut_ptr.offset(dst_offset), pixel_size);
                            }
                        }
                        EBitMapDataPtr::FloatPtr(src_ptr) => {
                            if let EBitMapDataMutPtr::FloatMutPtr(dst_mut_ptr) = dst
                            {
                                std::ptr::copy_nonoverlapping(src_ptr.offset(src_offset), dst_mut_ptr.offset(dst_offset), pixel_size);
                            }
                        }
                    }
                }
                dst_offset += pixel_size as isize;
            } // i
        } // j
    } // j

    cubemap
}

pub fn convert_equirectangle_to_cubemap_faces(bitmap: &GkBitMap) -> GkBitMap
{
    convert_vertical_cross_to_cubemap_faces(&convert_equirectangle_to_vertical_cross(bitmap))
}

pub fn convert_multi_file_to_cubemap_faces(files: &Vec<std::path::PathBuf>) -> Result<GkBitMap, String>
{
    if files.len() != 6 { return Err(String::from("convert_multi_file_to_cubemap_faces() needs 6 file paths!")); }

    let mut img_data: Vec<u8> = Vec::new();
    let mut img_width: u32 = 0;
    let mut img_height: u32 = 0;

    for file in files.iter()
    {
        let img_result = image::open(file.as_path());
        let img: DynamicImage;
        match img_result
        {
            Ok(i) => img = i,
            Err(e) => 
            {
                log_err!(e.to_string());
                return Err(e.to_string()); 
            }
        }

        let pixels = img.to_rgba8().into_raw();
        img_width = img.width();
        img_height = img.height();

        if img_data.capacity() != pixels.len() * 6 { img_data.reserve(pixels.len() * 6); }
        for i in 0..pixels.len()
        {
            img_data.push(pixels[i]);
        }
    }

    // *width = img_width;
    // *height = img_height;

    let result: GkBitMap = GkBitMap::new(img_width, img_height, Some(6), 4, EBitMapData::UByte(img_data));

    Ok(result)
}