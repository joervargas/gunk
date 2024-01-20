use nalgebra_glm as glm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EBitMapType
{
    BitMapType2D,
    BitMapTypeCube
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

// pub trait BitMapScalar: From<u8> + Scalar{}
// impl<T: From<u8> + Scalar> BitMapScalar for T{}

// pub struct BitMap<T: BitMapScalar>
// {
//     pub w: i32,
//     pub h: i32,
//     pub d: i32,
//     pub comp: usize,
//     pub map_type: EBitMapType,
//     pub data: Vec<T>
// }

// impl<T: BitMapScalar> BitMap<T>
// {
//     pub fn new(w: i32, h: i32, d: Option<i32>, comp: usize) -> Self
//     {
//         let (depth, map_type ) = if d.is_some() { (d.unwrap(), EBitMapType::BitMapTypeCube) } else { (1, EBitMapType::BitMapType2D) };
//         let size = (w * h * depth) as usize * comp * std::mem::size_of::<T>();
//         let mut data: Vec<T> = Vec::new();
//         data.resize(size, T::from(0));

//         Self { w, h, d: depth, comp, map_type, data }
//     }

//     fn set_pixel(&mut self, x: i32, y: i32, c: &TVec4<T>)
//     {
//         let offset = self.comp * ((y * self.w + x) as usize);

//         if self.comp > 0 { self.data[offset + 0] = c.x.clone(); }
//         if self.comp > 1 { self.data[offset + 1] = c.y.clone(); }
//         if self.comp > 2 { self.data[offset + 2] = c.z.clone(); }
//         if self.comp > 3 { self.data[offset + 3] = c.w.clone(); }
//     }

//     fn get_pixel(&self, x: i32, y: i32) -> TVec4<T>
//     {
//         let offset = self.comp * ((y * self.w + x) as usize);

//         let result: TVec4<T> = vec4::<T>(
//             if self.comp > 0 { self.data[offset + 0].clone() } else { T::from(0) },
//             if self.comp > 1 { self.data[offset + 1].clone() } else { T::from(0) },
//             if self.comp > 2 { self.data[offset + 2].clone() } else { T::from(0) },
//             if self.comp > 3 { self.data[offset + 3].clone() } else { T::from(0) }
//         );

//         result
//     }

//     pub fn get_bytes_per_component(&self) -> usize
//     {
//         std::mem::size_of::<T>()
//     }
// }

// pub trait BitMapScalar: From<u8> + Scalar{}
// impl<T: From<u8> + Scalar> BitMapScalar for T{}

#[derive(Clone)]
pub struct BitMap
{
    pub bm_type: EBitMapType,
    pub w: i32,
    pub h: i32,
    pub d: i32,
    pub comp: usize,
    pub data: Vec<u8>
}

impl BitMap
{
    pub fn new(w: i32, h: i32, d: Option<i32>, comp: usize, pixels: &Vec<u8>) -> Self
    {
        let d = if d.is_some() { d.unwrap() } else { 1 };
        let bm_type = if d == 6 { EBitMapType::BitMapTypeCube } else  { EBitMapType::BitMapType2D };

        let size = (w * h * d) as usize * comp;
        let mut data = Vec::new();
        data.resize(size, 0);

        if !pixels.is_empty()
        {
            data.copy_from_slice(pixels.as_slice());
            // unsafe { std::ptr::copy_nonoverlapping(pixels.as_ptr(), data.as_mut_ptr(), size); }
        }

        Self { bm_type, w, h, d, comp, data }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, c: &glm::Vec4)
    {
        let offset = self.comp * ((y * self.w + x) as usize);
        if self.comp > 0 { self.data[offset + 0] = (c.x * 255.0) as u8; }
        if self.comp > 1 { self.data[offset + 1] = (c.y * 255.0) as u8; }
        if self.comp > 2 { self.data[offset + 2] = (c.z * 255.0) as u8; }
        if self.comp > 3 { self.data[offset + 3] = (c.w * 255.0) as u8; }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> glm::Vec4
    {
        let offset = self.comp * ((y * self.w + x) as usize);

        let result = glm::Vec4::new(
            if self.comp > 0 { self.data[offset + 0].clone() as f32 / 255.0 } else { 0.0 },
            if self.comp > 1 { self.data[offset + 1].clone() as f32 / 255.0 } else { 0.0 },
            if self.comp > 2 { self.data[offset + 2].clone() as f32 / 255.0 } else { 0.0 },
            if self.comp > 3 { self.data[offset + 3].clone() as f32 / 255.0 } else { 0.0 }
        );
        result
    }

    pub fn get_bytes_per_component(&self) -> usize
    {
        std::mem::size_of::<u8>()
    }

}