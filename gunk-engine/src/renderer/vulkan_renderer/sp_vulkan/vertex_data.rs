use nalgebra_glm as glm;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VertexData
{
    // pub tc:     [f32; 2],
    // pub pos:    [f32; 3],
    pub pos:        [f32; 3],
    pub color:      [f32; 3],
    pub tex_coord:  [f32; 2]
}

impl VertexData
{
    // pub fn new(pos: glm::Vec3, text_coords: glm::Vec2) -> Self
    // {
    //     Self
    //     {
    //         tc: [text_coords.x, text_coords.y],
    //         pos: [pos.x, pos.y, pos.z]
    //     }
    // }

    pub fn new(pos: glm::Vec3, color: glm::Vec3, tex_coords: glm::Vec2) -> Self
    {
        Self
        {
            pos:        [pos.x, pos.y, pos.z],
            color:      [color.x, color.y, color.z],
            tex_coord:  [tex_coords.x, tex_coords.y]
        }
    }
}
