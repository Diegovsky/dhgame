use alloc::boxed::Box;
use alloc::format;

use crate::resources::{self, FileError};
use crate::sys::video_registers as vr;
use crate::{OAM, dma_copy_slice};

pub struct Texture {
    pub img: Box<[u8]>,
}

impl Texture {
    pub fn load(stem: &str) -> Result<Self, FileError> {
        let img = resources::read(&format!("{stem}"))?;
        Ok(Self { img })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteType {
    Sprites,
    Backgrounds,
}

pub struct Palette {
    pub data: Box<[u8]>,
}

impl Palette {
    pub fn load(path: &str) -> Result<Self, FileError> {
        let data = resources::read(path)?;
        Ok(Self { data })
    }
    pub fn write(&self, oam: &OAM, loc: PaletteType) {
        let is_sub = oam.is_sub();
        let dst = match (loc, is_sub) {
            (PaletteType::Sprites, true) => vr::SPRITE_PALETTE_SUB,
            (PaletteType::Sprites, false) => vr::SPRITE_PALETTE,
            (PaletteType::Backgrounds, true) => vr::BG_PALETTE_SUB,
            (PaletteType::Backgrounds, false) => vr::BG_PALETTE,
        };
        unsafe {
            dma_copy_slice(&*self.data, dst);
        }
    }
}
