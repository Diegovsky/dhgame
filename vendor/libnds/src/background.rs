use core::{
    ops::{Deref, Index, IndexMut, Range},
    ptr::NonNull,
};

use texture::Texture;

use super::*;
#[doc(alias = "bg")]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Background(pub(crate) i32);

impl Background {
    pub fn set_texture(self, texture: &Texture) {
        unsafe {
            dma_copy_slice(&*texture.img, self.raw_ptr());
        }
    }
    pub fn set_map(self, map: &[u8]) {
        unsafe {
            dma_copy_slice(map, self.raw_ptr());
        }
    }
    #[doc(alias = "bgGetGfxPtr")]
    pub fn raw_ptr(self) -> *mut u16 {
        unsafe { nds::bgGetGfxPtr(self.0) }
    }
    pub fn ptr(self) -> BackgroundPtr {
        BackgroundPtr(
            NonNull::new(self.raw_ptr() as *mut u8).expect("Got null pointer from bgGetMapPtr()"),
        )
    }
}

#[derive(Clone, Copy)]
pub struct BackgroundPtr(NonNull<u8>);

impl BackgroundPtr {
    fn parts_from_range(self, index: Range<usize>) -> (NonNull<u8>, usize) {
        fn assert_in_range(base: NonNull<u8>) {
            debug_assert!(
                NonNull::new(0x06000000 as *mut u8).unwrap() >= base
                    && base <= NonNull::new(0x0607F800 as *mut u8).unwrap()
            );
        }
        unsafe {
            let base = self.0.add(index.start);
            let end = self.0.add(index.end);
            assert_in_range(base);
            assert_in_range(end);
            (base, end.offset_from(base) as usize)
        }
    }
}

impl Index<Range<usize>> for BackgroundPtr {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        let (data, len) = self.parts_from_range(index);
        unsafe { core::slice::from_raw_parts(data.as_ptr(), len) }
    }
}

impl IndexMut<Range<usize>> for BackgroundPtr {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        let (data, len) = self.parts_from_range(index);
        unsafe { core::slice::from_raw_parts_mut(data.as_ptr(), len) }
    }
}

#[doc(alias = "bgUpdate")]
pub fn update() {
    unsafe {
        nds::bgUpdate();
    }
}

#[repr(i32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// 8bpp Tiled background with 16 bit tile indexes and no allowed rotation or scaling
    Text8bpp = 0,
    /// 4bpp Tiled background with 16 bit tile indexes and no allowed rotation or scaling
    Text4bpp = 1,
    /// Tiled background with 8 bit tile indexes Can be scaled and rotated
    Rotation = 2,
    /// Tiled background with 16 bit tile indexes Can be scaled and rotated
    ExRotation = 3,
    /// Bitmap background with 8 bit color values which index into a 256 color palette
    Bmp8 = 4,
    /// Bitmap background with 16 bit color values of the form aBBBBBGGGGGRRRRR (if 'a' is not set, the pixel will be transparent)
    Bmp16 = 5,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum RotSize {
    /// 128 x 128 pixel rotation background
    R128x128 = 0,
    /// 256 x 256 pixel rotation background
    R256x256 = 16384,
    /// 512 x 512 pixel rotation background
    R512x512 = 32768,
    /// 1024 x 1024 pixel rotation background
    R1024x1024 = 49152,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum TextSize {
    /// 256 x 256 pixel text background
    T256x256 = 65536,
    /// 512 x 256 pixel text background
    T512x256 = 81920,
    /// 256 x 512 pixel text background
    T256x512 = 98304,
    /// 512 x 512 pixel text background
    T512x512 = 114688,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ExtRotSize {
    /// 128 x 128 pixel extended rotation background
    ER_128x128 = 131072,
    /// 256 x 256 pixel extended rotation background
    ER_256x256 = 147456,
    /// 512 x 512 pixel extended rotation background
    ER_512x512 = 163840,
    /// 1024 x 1024 pixel extended rotation background
    ER_1024x1024 = 180224,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum Bitmap8Size {
    /// 128 x 128 pixel 8-bit bitmap background
    B8_128x128 = 196736,
    /// 256 x 256 pixel 8-bit bitmap background
    B8_256x256 = 213120,
    /// 512 x 256 pixel 8-bit bitmap background
    B8_512x256 = 229504,
    /// 512 x 512 pixel 8-bit bitmap background
    B8_512x512 = 245888,
    /// 1024 x 512 pixel 8-bit bitmap background
    B8_1024x512 = 212992,
    /// 512 x 1024 pixel 8-bit bitmap background
    B8_512x1024 = 196608,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum Bitmap16Size {
    /// 128 x 128 pixel 16-bit bitmap background
    B16_128x128 = 262276,
    /// 256 x 256 pixel 16-bit bitmap background
    B16_256x256 = 278660,
    /// 512 x 256 pixel 16-bit bitmap background
    B16_512x256 = 295044,
    /// 512 x 512 pixel 16-bit bitmap background
    B16_512x512 = 311428,
}

pub trait Size: Into<u32> {}

impl Size for Bitmap16Size {}
impl Size for Bitmap8Size {}
impl Size for ExtRotSize {}
impl Size for TextSize {}
