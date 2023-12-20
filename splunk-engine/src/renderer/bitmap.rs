use nalgebra_glm as glm;
use glm::{ Scalar, TVec4, vec4};

use std::convert::From;

pub enum EBitMapType
{
    BitMapType_2D,
    BitMapType_Cube
}

// enum EBitMapFormat
// {
//     BitMapFormat_UByte,
//     BitMapFormat_Float
// }

// struct BitMap
// {
//     w: i32,
//     h: i32,
//     d: i32,
//     comp: i32,
//     map_type: EBitMapType,
//     map_fmt: EBitMapFormat,
//     data: Vec<u8>
// }

pub trait BitMapScalar: From<u8> + Scalar{}
impl<T: From<u8> + Scalar> BitMapScalar for T{}

pub struct BitMap<T: BitMapScalar>
{
    pub w: i32,
    pub h: i32,
    pub d: i32,
    pub comp: usize,
    pub map_type: EBitMapType,
    pub data: Vec<T>
}

impl<T: BitMapScalar> BitMap<T>
{
    pub fn new(w: i32, h: i32, d: Option<i32>, comp: usize) -> Self
    {
        let (depth, map_type ) = if d.is_some() { (d.unwrap(), EBitMapType::BitMapType_Cube) } else { (1, EBitMapType::BitMapType_2D) };
        let size = (w * h * depth) as usize * comp * std::mem::size_of::<T>();
        let mut data: Vec<T> = Vec::new();
        data.resize(size, T::from(0));

        Self { w, h, d: depth, comp, map_type, data }
    }

    fn set_pixel(&mut self, x: i32, y: i32, c: &TVec4<T>)
    {
        let offset = self.comp * ((y * self.w + x) as usize);

        if self.comp > 0 { self.data[offset + 0] = c.x.clone(); }
        if self.comp > 1 { self.data[offset + 1] = c.y.clone(); }
        if self.comp > 2 { self.data[offset + 2] = c.z.clone(); }
        if self.comp > 3 { self.data[offset + 3] = c.w.clone(); }
    }

    fn get_pixel(&self, x: i32, y: i32) -> TVec4<T>
    {
        let offset = self.comp * ((y * self.w + x) as usize);

        let result: TVec4<T> = vec4::<T>(
            if self.comp > 0 { self.data[offset + 0].clone() } else { T::from(0) },
            if self.comp > 1 { self.data[offset + 1].clone() } else { T::from(0) },
            if self.comp > 2 { self.data[offset + 2].clone() } else { T::from(0) },
            if self.comp > 3 { self.data[offset + 3].clone() } else { T::from(0) }
        );

        result
    }

    pub fn get_bytes_per_component(&self) -> usize
    {
        std::mem::size_of::<T>()
    }
}