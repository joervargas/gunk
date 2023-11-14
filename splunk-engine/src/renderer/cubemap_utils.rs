use nalgebra_glm as glm;

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