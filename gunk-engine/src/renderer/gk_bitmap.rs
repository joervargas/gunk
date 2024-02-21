use nalgebra_glm as glm;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EBitMapType
{
    Type2D,
    TypeCube
}

#[derive(Clone, Debug, PartialEq)]
pub enum EBitMapData
{
    UByte(Vec<u8>),
    Float(Vec<f32>)
}

pub enum EBitMapDataPtr
{
    UBytePtr(*const u8),
    FloatPtr(*const f32)
}

pub enum EBitMapDataMutPtr
{
    UByteMutPtr(*mut u8),
    FloatMutPtr(*mut f32)
}

// impl EBitMapDataMutPtr
// {
//     pub fn offset(&mut self, count: isize)
// }

// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// enum EBitMapFormat
// {
//     FormatUByte,
//     FormatFloat
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

// #[derive(Clone)]
// pub struct SpBitMap
// {
//     pub bm_type: EBitMapType,
//     pub width: u32,
//     pub height: u32,
//     pub layers: u32,
//     pub channels: usize,
//     pub data: Vec<u8>
// }

// impl SpBitMap
// {
//     pub fn new(width: u32, height: u32, layers: Option<u32>, channels: usize, pixels: &Vec<u8>) -> Self
//     {
//         let layers = if layers.is_some() { layers.unwrap() } else { 1 };
//         let bm_type = if layers == 6 { EBitMapType::BitMapTypeCube } else  { EBitMapType::BitMapType2D };

//         let size = (width * height * layers) as usize * channels;
//         let mut data = Vec::new();
//         data.resize(size, 0);

//         if !pixels.is_empty()
//         {
//             data.copy_from_slice(pixels.as_slice());
//             // unsafe { std::ptr::copy_nonoverlapping(pixels.as_ptr(), data.as_mut_ptr(), size); }
//         }

//         Self { bm_type, width, height, layers, channels, data }
//     }

//     pub fn set_pixel(&mut self, x: i32, y: i32, c: &glm::Vec4)
//     {
//         let offset = self.channels * ((y * self.width as i32 + x) as usize);
//         if self.channels > 0 { self.data[offset + 0] = (c.x * 255.0) as u8; }
//         if self.channels > 1 { self.data[offset + 1] = (c.y * 255.0) as u8; }
//         if self.channels > 2 { self.data[offset + 2] = (c.z * 255.0) as u8; }
//         if self.channels > 3 { self.data[offset + 3] = (c.w * 255.0) as u8; }
//     }

//     pub fn get_pixel(&self, x: i32, y: i32) -> glm::Vec4
//     {
//         let offset = self.channels * ((y * self.width as i32 + x) as usize);

//         let result = glm::Vec4::new(
//             if self.channels > 0 { self.data[offset + 0].clone() as f32 / 255.0 } else { 0.0 },
//             if self.channels > 1 { self.data[offset + 1].clone() as f32 / 255.0 } else { 0.0 },
//             if self.channels > 2 { self.data[offset + 2].clone() as f32 / 255.0 } else { 0.0 },
//             if self.channels > 3 { self.data[offset + 3].clone() as f32 / 255.0 } else { 0.0 }
//         );
//         result
//     }

//     pub fn get_bytes_per_component(&self) -> usize
//     {
//         std::mem::size_of::<u8>()
//     }

// }

#[derive(Clone)]
pub struct GkBitMap
{
    pub bm_type: EBitMapType,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub channels: usize,
    pub data: EBitMapData,
}

impl GkBitMap
{
    pub fn new(width: u32, height: u32, layers: Option<u32>, channels: usize, data: EBitMapData) -> Self
    {
        let layers = if layers.is_some() { layers.unwrap() } else { 1 };
        let bm_type = if layers == 6 { EBitMapType::TypeCube } else  { EBitMapType::Type2D };

        // let size = (width * height * layers) as usize * channels;
        // let mut data = Vec::new();
        // data.resize(size, 0);
        
        // let d = match data{
        //     EBitMapData::UByte(udata) => 
        //     {
        //         let mut u : Vec<u8> = Vec::new();
        //         if !udata.is_empty()
        //         {
        //             u.copy_from_slice(udata.as_slice());
        //             unsafe { std::ptr::copy_nonoverlapping(udata.as_ptr(), u.as_mut_ptr(), udata.len()); }
        //         }
        //         EBitMapData::UByte(u)
        //     },
        //     EBitMapData::Float(fdata) => 
        //     {
        //         let mut f : Vec<f32> = Vec::new();
        //         if !fdata.is_empty()
        //         {
        //             f.copy_from_slice(fdata.as_slice());
        //             unsafe { std::ptr::copy_nonoverlapping(fdata.as_ptr(), f.as_mut_ptr(), fdata.len()); }
        //         }
        //         EBitMapData::Float(f)
        //     } 
        // };

        Self { bm_type, width, height, layers, channels, data }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, c: &glm::Vec4)
    {
        let offset = self.channels * ((y * self.width as i32 + x) as usize);
        match self.data {
            EBitMapData::UByte(ref mut udata) =>
            {
                if self.channels > 0 { udata[offset + 0] = (c.x * 255.0) as u8; }
                if self.channels > 1 { udata[offset + 1] = (c.y * 255.0) as u8; }
                if self.channels > 2 { udata[offset + 2] = (c.z * 255.0) as u8; }
                if self.channels > 3 { udata[offset + 3] = (c.w * 255.0) as u8; }
            },
            EBitMapData::Float(ref mut fdata) =>
            {
                if self.channels > 0 { fdata[offset + 0] = c.x; }
                if self.channels > 1 { fdata[offset + 1] = c.y; }
                if self.channels > 2 { fdata[offset + 2] = c.z; }
                if self.channels > 3 { fdata[offset + 3] = c.w; }
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> glm::Vec4
    {
        let offset = self.channels * ((y * self.width as i32 + x) as usize);

        let result = match self.data
        {
            EBitMapData::UByte(ref udata) => 
            {
                glm::Vec4::new(
                    if self.channels > 0 { udata[offset + 0].clone() as f32 / 255.0 } else { 0.0 },
                    if self.channels > 1 { udata[offset + 1].clone() as f32 / 255.0 } else { 0.0 },
                    if self.channels > 2 { udata[offset + 2].clone() as f32 / 255.0 } else { 0.0 },
                    if self.channels > 3 { udata[offset + 3].clone() as f32 / 255.0 } else { 0.0 }
                )
            },
            EBitMapData::Float(ref fdata) =>
            {
                glm::Vec4::new(
                    if self.channels > 0 { fdata[offset + 0].clone() } else { 0.0 },
                    if self.channels > 1 { fdata[offset + 1].clone() } else { 0.0 },
                    if self.channels > 2 { fdata[offset + 2].clone() } else { 0.0 },
                    if self.channels > 3 { fdata[offset + 3].clone() } else { 0.0 }
                )
            }
        };

        result
    }

    pub fn get_bytes_per_component(&self) -> usize
    {
        match self.data
        {
            EBitMapData::UByte(_) => std::mem::size_of::<u8>(),
            EBitMapData::Float(_) => std::mem::size_of::<f32>()
        }
    }

    pub fn get_data_ptr(&self) -> EBitMapDataPtr
    {
        match self.data
        {
            EBitMapData::UByte(ref udata) => EBitMapDataPtr::UBytePtr(udata.as_ptr()),
            EBitMapData::Float(ref fdata) => EBitMapDataPtr::FloatPtr(fdata.as_ptr())
        }
    }

    pub fn get_data_mut_ptr(&mut self) -> EBitMapDataMutPtr
    {
        match self.data
        {
            EBitMapData::UByte(ref mut udata) => EBitMapDataMutPtr::UByteMutPtr(udata.as_mut_ptr()),
            EBitMapData::Float(ref mut fdata) => EBitMapDataMutPtr::FloatMutPtr(fdata.as_mut_ptr())
        }
    }

}