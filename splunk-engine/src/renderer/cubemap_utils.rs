use glm::pi;
use image::{DynamicImage, imageops::FilterType};
use nalgebra_glm as glm;

use crate::renderer::bitmap::{BitMap, BitMapScalar};


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

pub fn convolve_diffuse<T: BitMapScalar>(img: &DynamicImage, src_w: i32, src_h: i32, dst_w: i32, dst_h: i32, num_monte_carlo_samples: i32, output: &mut glm::Vec3 )
{
    assert!(src_w == 2 * src_h);
    if src_w != 2 * src_h { return; }

    let mut tmp : Vec<glm::Vec3> = Vec::new();
    tmp.reserve((dst_w * dst_h) as usize);

    let new_img = img.resize(dst_w as u32, dst_h as u32, FilterType::Nearest);
    
    let src_w = dst_w;
    let src_h = dst_h;

    for y in 0..dst_h
    {
        print!("Line {} ...\n", y);
        let theta1 : f32 = y as f32 / (dst_h as f32) * std::f32::consts::PI;

        for x in 0..dst_w
        {
            let phi1 = x as f32 / (dst_w as f32) * (std::f32::consts::PI * 2.0);
            let v1 = glm::vec3(theta1.cos() * phi1.cos(), theta1.sin() * phi1.sin(), theta1.cos());

            let mut color = glm::Vec3::new(0.0, 0.0, 0.0);
            let mut weight: f32 = 0.0;

            for i in 0..num_monte_carlo_samples
            {
                let h = hammersley_2d(i as u32, num_monte_carlo_samples as u32);
                let x1 = (h.x * src_w as f32).floor() as i32;
                let y1 = (h.y * src_h as f32).floor() as i32;

                let theta2 : f32 = (y1 as f32) / (src_h as f32) * std::f32::consts::PI;
                let phi2 : f32 = (x1 as f32) / (src_w as f32) * (std::f32::consts::PI * 2.0);
                let v2 = glm::vec3(theta2.sin() * phi2.cos(), theta2.sin() * phi2.sin(), theta2.cos());
                let d : f32 = 0.0_f32.max(glm::dot(&v1, &v2));
                if d > 0.01
                {
                    color += tmp[(y1 * src_w + x1) as usize] * d;
                    weight += d;
                }
            }
            output[(y * dst_w + x) as usize] = color / weight;
        }
     }

    todo!()
}

pub fn convert_equirectangle_to_vertical_cross<T: BitMapScalar>(bitmap: &BitMap<T>) -> BitMap<T>
{
    todo!()
}

pub fn convert_vertical_cross_to_cubemap_faces<T: BitMapScalar>(bitmap: &BitMap<T>) -> BitMap<T>
{
    todo!();
}

pub fn convert_equirectangle_to_cubemap_faces<T: BitMapScalar>(bitmap: &BitMap<T>) -> BitMap<T>
{
    convert_vertical_cross_to_cubemap_faces(&convert_equirectangle_to_vertical_cross(bitmap))
}