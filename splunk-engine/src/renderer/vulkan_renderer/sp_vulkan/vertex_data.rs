use nalgebra_glm as glm;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VertexData
{
    pub tc:     [f32; 2],
    pub pos:    [f32; 3],
}

impl VertexData
{
    pub fn new(pos: glm::Vec3, text_coords: glm::Vec2) -> Self
    {
        Self
        {
            tc: [text_coords.x, text_coords.y],
            pos: [pos.x, pos.y, pos.z]
        }
    }
}