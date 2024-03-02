
use nalgebra_glm as glm;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EBitMapType
{
    Type2D,
    TypeCube
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EBitMapFormat
{
    UByte,
    Float
}

// #[derive(Clone, Debug, PartialEq)]
// pub enum EBitMapData
// {
//     UByte(Vec<u8>),
//     Float(Vec<f32>)
// }

// pub enum EBitMapDataPtr
// {
//     UBytePtr(*const u8),
//     FloatPtr(*const f32)
// }

// pub enum EBitMapDataMutPtr
// {
//     UByteMutPtr(*mut u8),
//     FloatMutPtr(*mut f32)
// }



// #[derive(Clone)]
// pub struct GkBitMap
// {
//     pub bm_type: EBitMapType,
//     pub width: u32,
//     pub height: u32,
//     pub layers: u32,
//     pub channels: usize,
//     pub data: EBitMapData,
// }

// impl GkBitMap
// {
//     pub fn new(width: u32, height: u32, layers: Option<u32>, channels: usize, data: EBitMapData) -> Self
//     {
//         let layers = if layers.is_some() { layers.unwrap() } else { 1 };
//         let bm_type = if layers == 6 { EBitMapType::TypeCube } else  { EBitMapType::Type2D };

//         let size = (width * height * layers) as usize * channels;
//         // Self::check_data_size(&mut data, size);
        
//         // let mut data = Vec::new();
//         // data.resize(size, 0);
        
//         let d = match data{
//             EBitMapData::UByte(udata) => 
//             {
//                 let mut u : Vec<u8> = Vec::new();
//                 if !udata.is_empty()
//                 {
//                     u.resize(udata.len(), 0);
//                     u.copy_from_slice(udata.as_slice());
//                     // unsafe { std::ptr::copy_nonoverlapping(udata.as_ptr(), u.as_mut_ptr(), udata.len()); }
//                 } else {
//                     u.resize(size * std::mem::size_of::<f32>(), 0);
//                 }
//                 EBitMapData::UByte(u)
//             },
//             EBitMapData::Float(fdata) => 
//             {
//                 let mut f : Vec<f32> = Vec::new();
//                 if !fdata.is_empty()
//                 {
//                     f.resize(fdata.len(), 0.0);
//                     f.copy_from_slice(fdata.as_slice());
//                     // unsafe { std::ptr::copy_nonoverlapping(fdata.as_ptr(), f.as_mut_ptr(), fdata.len()); }
//                 } else {
//                     f.resize(size, 0.0);
//                 }
//                 EBitMapData::Float(f)
//             } 
//         };

//         Self { bm_type, width, height, layers, channels, data: d }
//     }

//     pub fn set_pixel(&mut self, x: i32, y: i32, c: &glm::Vec4)
//     {
//         let offset = self.channels * ((y * self.width as i32 + x) as usize);
//         match self.data {
//             EBitMapData::UByte(ref mut udata) =>
//             {
//                 if self.channels > 0 { udata[offset + 0] = (c.x * 255.0) as u8; }
//                 if self.channels > 1 { udata[offset + 1] = (c.y * 255.0) as u8; }
//                 if self.channels > 2 { udata[offset + 2] = (c.z * 255.0) as u8; }
//                 if self.channels > 3 { udata[offset + 3] = (c.w * 255.0) as u8; }
//             },
//             EBitMapData::Float(ref mut fdata) =>
//             {
//                 if self.channels > 0 { fdata[offset + 0] = c.x; }
//                 if self.channels > 1 { fdata[offset + 1] = c.y; }
//                 if self.channels > 2 { fdata[offset + 2] = c.z; }
//                 if self.channels > 3 { fdata[offset + 3] = c.w; }
//             }
//         }
//     }

//     pub fn get_pixel(&self, x: i32, y: i32) -> glm::Vec4
//     {
//         let offset = self.channels * ((y * self.width as i32 + x) as usize);

//         let result = match self.data
//         {
//             EBitMapData::UByte(ref udata) => 
//             {
//                 glm::Vec4::new(
//                     if self.channels > 0 { udata[offset + 0].clone() as f32 / 255.0 } else { 0.0 },
//                     if self.channels > 1 { udata[offset + 1].clone() as f32 / 255.0 } else { 0.0 },
//                     if self.channels > 2 { udata[offset + 2].clone() as f32 / 255.0 } else { 0.0 },
//                     if self.channels > 3 { udata[offset + 3].clone() as f32 / 255.0 } else { 0.0 }
//                 )
//             },
//             EBitMapData::Float(ref fdata) =>
//             {
//                 glm::Vec4::new(
//                     if self.channels > 0 { fdata[offset + 0].clone() } else { 0.0 },
//                     if self.channels > 1 { fdata[offset + 1].clone() } else { 0.0 },
//                     if self.channels > 2 { fdata[offset + 2].clone() } else { 0.0 },
//                     if self.channels > 3 { fdata[offset + 3].clone() } else { 0.0 }
//                 )
//             }
//         };

//         result
//     }

//     pub fn get_bytes_per_component(&self) -> usize
//     {
//         match self.data
//         {
//             EBitMapData::UByte(_) => std::mem::size_of::<u8>(),
//             EBitMapData::Float(_) => std::mem::size_of::<f32>()
//         }
//     }

//     pub fn get_data_ptr(&self) -> EBitMapDataPtr
//     {
//         match self.data
//         {
//             EBitMapData::UByte(ref udata) => EBitMapDataPtr::UBytePtr(udata.as_ptr()),
//             EBitMapData::Float(ref fdata) => EBitMapDataPtr::FloatPtr(fdata.as_ptr())
//         }
//     }

//     pub fn get_data_mut_ptr(&mut self) -> EBitMapDataMutPtr
//     {
//         match self.data
//         {
//             EBitMapData::UByte(ref mut udata) => EBitMapDataMutPtr::UByteMutPtr(udata.as_mut_ptr()),
//             EBitMapData::Float(ref mut fdata) => EBitMapDataMutPtr::FloatMutPtr(fdata.as_mut_ptr())
//         }
//     }

#[derive(Clone)]
pub struct GkBitMap
{
    pub bm_type: EBitMapType,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub channels: usize,
    pub format: EBitMapFormat,
    pub data: Vec<u8>,
}

impl GkBitMap
{
    pub fn new(width: u32, height: u32, layers: u32, channels: usize, format: EBitMapFormat, data: Vec<u8>) -> Self
    {
        let bm_type = if layers == 6 { EBitMapType::TypeCube } else  { EBitMapType::Type2D };

        let size = (width * height * layers) as usize * Self::get_bytes_per_component(&format) * channels;
        
        // let mut d = Vec::with_capacity(size);
        // if !data.is_empty()
        // {
        //     d.copy_from_slice(data.as_slice());
        // }
        let d = if !data.is_empty()
        {
            let mut v = Vec::new();
            v.resize(data.len(), 0);
            v.copy_from_slice(data.as_slice());
            v
        } else {
            let mut v: Vec<u8> = Vec::new();
            v.resize(size, 0);
            v
        };

        Self { bm_type, width, height, layers, channels, format, data: d }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, c: &glm::Vec4)
    {
        let offset = self.channels * ((y * self.width as i32 + x) as usize);
        match self.format
        {
            EBitMapFormat::UByte => {
                if self.channels > 0 { self.data[offset + 0] = (c.x * 255.0) as u8; }
                if self.channels > 1 { self.data[offset + 1] = (c.y * 255.0) as u8; }
                if self.channels > 2 { self.data[offset + 2] = (c.z * 255.0) as u8; }
                if self.channels > 3 { self.data[offset + 3] = (c.w * 255.0) as u8; }
            },
            EBitMapFormat::Float => {
                // let channel_size = Self::get_bytes_per_component(&self.format);
                // let dst_ptr = self.data.as_mut_ptr();
                // unsafe{
                //     if self.channels > 0 { std::ptr::copy_nonoverlapping(c.x.to_be_bytes().as_ptr(), dst_ptr.offset((offset + (0 * channel_size)) as isize), channel_size); }
                //     if self.channels > 1 { std::ptr::copy_nonoverlapping(c.y.to_be_bytes().as_ptr(), dst_ptr.offset((offset + (1 * channel_size)) as isize), channel_size); }
                //     if self.channels > 2 { std::ptr::copy_nonoverlapping(c.y.to_be_bytes().as_ptr(), dst_ptr.offset((offset + (2 * channel_size)) as isize), channel_size); }
                //     if self.channels > 3 { std::ptr::copy_nonoverlapping(c.y.to_be_bytes().as_ptr(), dst_ptr.offset((offset + (3 * channel_size)) as isize), channel_size); }
                // }
                let data = self.data.as_mut_ptr() as *mut f32;
                unsafe {
                    if self.channels > 0 { *data.offset(offset as isize + 0) = c.x; }
                    if self.channels > 1 { *data.offset(offset as isize + 1) = c.y; }
                    if self.channels > 2 { *data.offset(offset as isize + 2) = c.z; }
                    if self.channels > 3 { *data.offset(offset as isize + 3) = c.w; }
                }
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> glm::Vec4
    {
        let offset = self.channels * ((y * self.width as i32 + x) as usize);
        let result = match self.format {
            EBitMapFormat::UByte => {
                glm::Vec4::new(
                    if self.channels > 0 { (self.data[offset + 0] as f32) / 255.0 } else { 0.0 },
                    if self.channels > 1 { (self.data[offset + 1] as f32) / 255.0 } else { 0.0 },
                    if self.channels > 2 { (self.data[offset + 2] as f32) / 255.0 } else { 0.0 },
                    if self.channels > 3 { (self.data[offset + 3] as f32) / 255.0 } else { 0.0 }
                )
            },
            EBitMapFormat::Float => {
                // let channel_size = Self::get_bytes_per_component(&self.format);
                // glm::Vec4::new(
                //     if self.channels > 0 { f32::from_be_bytes( self.data.as_slice()[(offset + (0 * channel_size))..((offset + (0 * channel_size)) + 4)].try_into().expect("Failed to convert byte to float!") ) } else { 0.0 },
                //     if self.channels > 1 { f32::from_be_bytes( self.data.as_slice()[(offset + (1 * channel_size))..((offset + (1 * channel_size)) + 4)].try_into().expect("Failed to convert byte to float!") ) } else { 0.0 },
                //     if self.channels > 2 { f32::from_be_bytes( self.data.as_slice()[(offset + (2 * channel_size))..((offset + (2 * channel_size)) + 4)].try_into().expect("Failed to convert byte to float!") ) } else { 0.0 },
                //     if self.channels > 3 { f32::from_be_bytes( self.data.as_slice()[(offset + (3 * channel_size))..((offset + (3 * channel_size)) + 4)].try_into().expect("Failed to convert byte to float!") ) } else { 0.0 },
                // )
                let data = self.data.as_ptr() as *const f32;
                glm::Vec4::new(
                    if self.channels > 0 { unsafe { *data.offset(offset as isize + 0) } } else { 0.0 },
                    if self.channels > 1 { unsafe { *data.offset(offset as isize + 1) } } else { 0.0 },
                    if self.channels > 2 { unsafe { *data.offset(offset as isize + 2) } } else { 0.0 },
                    if self.channels > 3 { unsafe { *data.offset(offset as isize + 3) } } else { 0.0 },
                )
            }
        };
        result
    }

    pub fn get_bytes_per_component(format: &EBitMapFormat) -> usize
    {
        match format
        {
            EBitMapFormat::UByte => std::mem::size_of::<u8>(),
            EBitMapFormat::Float => std::mem::size_of::<f32>()
        }
    }

}