#![no_std]
use background::Background;
use bitflags::bitflags;
use core::ffi::c_int;
use int_enum::IntEnum;
pub use libnds_sys as sys;
pub(crate) use libnds_sys::arm9_bindings as nds;
use libnds_sys::{arm9_bindings::COPY_MODE_FILL, eprintln};
extern crate alloc;

pub mod background;
pub mod resources;
pub mod texture;
pub mod video;

pub unsafe fn dma_copy<M: Copy>(src: *const M, dst: *mut M) {
    unsafe {
        nds::dmaCopy(src as *const _, dst as *mut _, size_of::<M>() as u32);
    }
}

pub unsafe fn dma_copy_slice<M: Copy>(src: &[M], dst: *mut u16) {
    unsafe {
        eprintln!(
            "Copying {} bytes from 0x{:x} to 0x{:x}",
            (src.len() * size_of::<M>()),
            src.as_ptr() as usize,
            dst as usize
        );
        nds::dmaCopy(
            src.as_ptr() as *const _,
            dst as *mut _,
            (src.len() * size_of::<M>()) as u32,
        );
    }
}
pub fn fill_slice_u8(src: u8, dst: &mut [u8]) {
    unsafe {
        const S: usize = size_of::<u32>();
        let i = [src; S];
        assert_eq!(dst.len() % S, 0);
        let dst = core::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut _, dst.len() / S);
        fill_slice(u32::from_ne_bytes(i), dst)
    }
}

pub unsafe fn fill_slice(src: u32, dst: &mut [u32]) {
    unsafe {
        nds::swiFastCopy(
            &src as *const _ as *const _,
            dst.as_ptr() as *mut _,
            (dst.len() as u32 | COPY_MODE_FILL) as i32,
        );
    }
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSize {
    S8x8 = 2,
    S16x16 = 16392,
    S32x32 = 32800,
    S64x64 = 49280,
    S16x8 = 4100,
    S32x8 = 20488,
    S32x16 = 36880,
    S64x32 = 53312,
    S8x16 = 8196,
    S8x32 = 24584,
    S16x32 = 40976,
    S32x64 = 57408,
    SInvalid = 0,
}

impl SpriteSize {
    pub const fn height(&self) -> u8 {
        match self {
            SpriteSize::S8x8 | SpriteSize::S16x8 | SpriteSize::S32x8 => 8,
            SpriteSize::S8x16 | SpriteSize::S16x16 | SpriteSize::S32x16 => 16,
            SpriteSize::S32x32 | SpriteSize::S64x32 | SpriteSize::S8x32 | SpriteSize::S16x32 => 32,
            SpriteSize::S32x64 | SpriteSize::S64x64 => 64,
            SpriteSize::SInvalid => todo!(),
        }
    }
    pub const fn width(&self) -> u8 {
        match self {
            SpriteSize::S64x32 | SpriteSize::S64x64 => 64,
            SpriteSize::S32x16 | SpriteSize::S32x32 | SpriteSize::S32x64 | SpriteSize::S32x8 => 32,
            SpriteSize::S16x16 | SpriteSize::S16x32 | SpriteSize::S16x8 => 16,
            SpriteSize::S8x16 | SpriteSize::S8x32 | SpriteSize::S8x8 => 8,
            SpriteSize::SInvalid => todo!(),
        }
    }
    pub const fn size(&self) -> u16 {
        self.width() as u16 * self.height() as u16
    }
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum SpriteMapping {
    SM1D32 = 16,
    SM1D64 = 1048593,
    SM1D128 = 2097170,
    SM1D256 = 3145747,
    SM2D = 0,
    SMBmp1D128 = 2097234,
    SMBmp1D256 = 7340115,
    SMBmp2D128 = 2,
    SMBmp2D256 = 35,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum SpriteColorFormat {
    SP16Color = 0,
    SP256Color = 1,
    SPBmp = 3,
}

#[repr(i32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum SpriteMode {
    Normal = 0,
    Blended = 1,
    Windowed = 2,
    Bitmap = 3,
}
// pub trait Zeroed: Sized {
//     fn zeroed() -> Self {
//         unsafe { core::mem::zeroed() }
//     }
// }

// impl<T: Copy> Zeroed for T {}

#[derive(PartialEq, Eq)]
pub struct Gfx {
    gfx: *mut u16,
    oam: OAM,
    size: SpriteSize,
    format: SpriteColorFormat,
}

impl Gfx {
    pub fn set_texture(&self, data: &[u8]) {
        unsafe {
            let pixel_count = self.size.size() as usize;
            let size = pixel_count
                / if self.format == SpriteColorFormat::SP256Color {
                    1
                } else {
                    2
                };
            assert_eq!(data.len(), size);
            dma_copy_slice(data, self.gfx);
        }
    }
    pub const fn size(&self) -> SpriteSize {
        self.size
    }
}

impl Drop for Gfx {
    fn drop(&mut self) {
        unsafe {
            nds::oamFreeGfx(self.oam.0, self.gfx as *const _);
        }
    }
}

pub struct SpriteConfig {
    pub x: u8,
    pub y: u8,
    pub priority: c_int,
    pub palette_alpha: c_int,
    pub affine_index: c_int,
    pub size_double: bool,
    pub hide: bool,
    pub hflip: bool,
    pub vflip: bool,
    pub mosaic: bool,
}

impl Default for SpriteConfig {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            priority: 0,
            palette_alpha: 0,
            affine_index: -1,
            size_double: false,
            hide: false,
            hflip: false,
            vflip: false,
            mosaic: false,
        }
    }
}

use c2rust_bitfields::BitfieldStruct;

#[repr(C, packed)]
#[derive(BitfieldStruct)]
pub struct SpriteEntry {
    #[bitfield(name = "y", ty = "u8", bits = "0..=7")]
    #[bitfield(name = "rotate_scale", ty = "bool", bits = "8..=8")]
    #[bitfield(name = "double_size", ty = "bool", bits = "9..=9")]
    #[bitfield(name = "is_hidden", ty = "bool", bits = "9..=9")]
    #[bitfield(name = "obj_mode", ty = "u8", bits = "10..=11")]
    #[bitfield(name = "mosaic", ty = "bool", bits = "12..=12")]
    #[bitfield(name = "color_mode", ty = "bool", bits = "13..=13")]
    #[bitfield(name = "shape", ty = "u8", bits = "14..=15")]
    attr0: [u8; 2],

    #[bitfield(name = "x", ty = "u16", bits = "0..=8")]
    #[bitfield(name = "rotation_index", ty = "u8", bits = "9..=13")]
    #[bitfield(name = "h_flip", ty = "bool", bits = "12..=12")]
    #[bitfield(name = "v_flip", ty = "bool", bits = "13..=13")]
    #[bitfield(name = "size", ty = "u8", bits = "14..=15")]
    attr1: [u8; 2],

    #[bitfield(name = "tile_index", ty = "u16", bits = "0..=9")]
    #[bitfield(name = "priority", ty = "u8", bits = "10..=11")]
    #[bitfield(name = "palette", ty = "u8", bits = "12..=15")]
    attr2: [u8; 2],

    _pad: u16,
}

impl SpriteEntry {
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.rotate_scale() && !self.is_hidden()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OAM(*mut nds::OamState);

impl OAM {
    pub const fn main() -> OAM {
        OAM(&raw mut nds::oamMain)
    }

    pub const fn sub() -> OAM {
        OAM(&raw mut nds::oamSub)
    }

    pub fn init(self, mapping: SpriteMapping, extended_palette: bool) {
        unsafe {
            nds::oamInit(self.0, mapping.into(), extended_palette);
        }
    }
    pub fn is_sub(self) -> bool {
        self.0 == &raw mut nds::oamSub
    }

    #[doc(alias = "bgInit")]
    pub fn allocate_bg(
        self,
        layer: background::Layer,
        type_: background::Type,
        size: impl background::Size,
        map_base: i32,
        tile_base: i32,
    ) -> Background {
        let size = size.into();
        let bg_id = if !self.is_sub() {
            unsafe {
                nds::bgInit(
                    layer.into(),
                    type_.into(),
                    size,
                    map_base.into(),
                    tile_base.into(),
                )
            }
        } else {
            unsafe {
                nds::bgInitSub(
                    layer.into(),
                    type_.into(),
                    size,
                    map_base.into(),
                    tile_base.into(),
                )
            }
        };
        Background(bg_id)
    }

    pub fn allocate_gfx(self, size: SpriteSize, format: SpriteColorFormat) -> Gfx {
        Gfx {
            gfx: unsafe { nds::oamAllocateGfx(self.0, size.into(), format.into()) },
            size,
            format,
            oam: self,
        }
    }

    pub fn enable(self) {
        unsafe {
            nds::oamEnable(self.0);
        }
    }
    pub fn disable(self) {
        unsafe {
            nds::oamDisable(self.0);
        }
    }

    pub fn set_sprite(self, index: u8, gfx: &Gfx, sprite: &SpriteConfig) {
        let SpriteConfig {
            x,
            y,
            priority,
            palette_alpha,
            affine_index,
            size_double,
            hide,
            hflip,
            vflip,
            mosaic,
        } = *sprite;
        let Gfx {
            gfx, size, format, ..
        } = gfx;
        unsafe {
            nds::oamSet(
                self.0,
                index as _,
                x.into(),
                y.into(),
                priority,
                palette_alpha,
                (*size).into(),
                (*format).into(),
                *gfx as *const _,
                affine_index,
                size_double,
                hide,
                hflip,
                vflip,
                mosaic,
            );
        }
    }

    #[doc(alias = "oamSetXY")]
    pub fn set_sprite_pos(self, id: u8, x: u8, y: u8) {
        unsafe {
            nds::oamSetXY(self.0, id as _, x as _, y as _);
        }
    }

    #[doc(alias = "oamSetGfx")]
    pub fn set_sprite_gfx(self, id: u8, gfx: &Gfx) {
        let Gfx {
            gfx, size, format, ..
        } = *gfx;
        unsafe {
            nds::oamSetGfx(
                self.0,
                id.into(),
                size.into(),
                format.into(),
                gfx as *const _,
            );
        }
    }

    #[doc(alias = "oamSetHidden", alias = "disable sprite")]
    pub fn set_sprite_hidden(self, id: u8, hidden: bool) {
        unsafe {
            nds::oamSetHidden(self.0, id.into(), hidden);
        }
    }

    pub fn sprites(self) -> &'static mut [SpriteEntry] {
        unsafe {
            let ptr = (*self.0).__bindgen_anon_1.oamMemory;
            core::slice::from_raw_parts_mut(ptr.cast::<SpriteEntry>(), 128)
        }
    }

    #[doc(alias = "is sprite hidden")]
    pub fn is_sprite_hidden(self, id: u8, hidden: bool) {
        unsafe {
            let mem = (*self.0).__bindgen_anon_1.oamMemory;
        }
    }

    pub fn update(self) {
        unsafe {
            nds::oamUpdate(self.0);
        }
    }
}

pub fn wait_for_vblank() {
    unsafe {
        nds::swiWaitForVBlank();
    }
}

pub fn scan_keys() {
    unsafe {
        nds::scanKeys();
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct Keys: u16 {
        /// Keypad A button.
        const A = 1;
        /// Keypad B button.
        const B = 2;
        /// Keypad SELECT button.
        const SELECT = 4;
        /// Keypad START button.
        const START = 8;
        /// Keypad RIGHT button.
        const RIGHT = 16;
        /// Keypad LEFT button.
        const LEFT = 32;
        /// Keypad UP button.
        const UP = 64;
        /// Keypad DOWN button.
        const DOWN = 128;
        /// Right shoulder button.
        const R = 256;
        /// Left shoulder button.
        const L = 512;
        /// Keypad X button.
        const X = 1024;
        /// Keypad Y button.
        const Y = 2048;
        /// Touchscreen pendown.
        const TOUCH = 4096;
        /// Lid state.
        const LID = 8192;
        /// Debug button.
        const KEY_DEBUG = 16384;
    }
}

#[doc(alias = "keysHeld", alias = "scanKeys")]
pub fn held_keys() -> Keys {
    scan_keys();
    unsafe { Keys::from_bits(nds::keysHeld() as u16).unwrap() }
}
