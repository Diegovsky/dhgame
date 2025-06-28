use int_enum::IntEnum;

use crate::nds;

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum Mode2D {
    Mode0 = 65536,
    Mode1 = 65537,
    Mode2 = 65538,
    Mode3 = 65539,
    Mode4 = 65540,
    Mode5 = 65541,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum Mode3D {
    Mode0 = 65800,
    Mode1 = 65801,
    Mode2 = 65802,
    Mode3 = 65803,
    Mode4 = 65804,
    Mode5 = 65805,
    Mode6 = 65806,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum ModeOther {
    ModeFIFO = 196608,
    ModeFB0 = 131072,
    ModeFB1 = 393216,
    ModeFB2 = 655360,
    ModeFB3 = 917504,
}

pub trait ModeMain: Into<u32> {}
pub trait ModeSub: Into<u32> {}

impl ModeMain for Mode2D {}
impl ModeMain for Mode3D {}
impl ModeMain for ModeOther {}
impl ModeSub for Mode2D {}
impl ModeSub for ModeOther {}

pub fn set_main(video_mode: impl ModeMain) {
    unsafe {
        nds::videoSetMode(video_mode.into());
    }
}
pub fn set_sub(video_mode: impl ModeSub) {
    unsafe {
        nds::videoSetModeSub(video_mode.into());
    }
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum VRamTypeA {
    LCD = 0,
    // Will be at location 0x06000000
    MainBg0 = 1,
    // Will be at location 0x06020000
    MainBg1 = 9,
    // Will be at location 0x06040000
    MainBg2 = 17,
    // Will be at location 0x06060000
    MainBg3 = 25,
    // Will be at 0x06400000
    MainSprite0 = 2,
    // Will be at 0x06420000
    MainSprite1 = 10,
    TextureSlot0 = 3,
    TextureSlot1 = 11,
    TextureSlot2 = 19,
    TextureSlot3 = 27,
}
pub type VRamTypeB = VRamTypeA;

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum VRamTypeC {
    LCD = 0,
    // Will be at location 0x06000000
    MainBg0 = 1,
    // Will be at location 0x06020000
    MainBg1 = 9,
    // Will be at location 0x06040000
    MainBg2 = 17,
    // Will be at location 0x06060000
    MainBg3 = 25,
    // Will be at location 0x06000000
    ARM7Slot1 = 2,
    // Will be at location 0x06020000
    ARM7Slot2 = 10,
    // sub engine background slot 0.
    SubBg = 4,
    TextureSlot0 = 3,
    TextureSlot1 = 11,
    TextureSlot2 = 19,
    TextureSlot3 = 27,
}

#[repr(u32)]
#[derive(IntEnum, Clone, Copy, PartialEq, Eq)]
pub enum VRamTypeD {
    LCD = 0,
    // Will be at location 0x06000000
    MainBg0 = 1,
    // Will be at location 0x06020000
    MainBg1 = 9,
    // Will be at location 0x06040000
    MainBg2 = 17,
    // Will be at location 0x06060000
    MainBg3 = 25,
    // Will be at location 0x06000000
    ARM7Slot1 = 2,
    // Will be at location 0x06020000
    ARM7Slot2 = 10,
    // sub engine sprite slot 0.
    SubSprite = 4,
    TextureSlot0 = 3,
    TextureSlot1 = 11,
    TextureSlot2 = 19,
    TextureSlot3 = 27,
}
macro_rules! def_mod {
    ($ident:ident::$name:ident($ty:ty)) => {
        #[doc=concat!("Represents bank ", stringify!($ident), " as a Rust type.")]
        pub struct $ident;
        impl $ident {
            #[doc=concat!("Sets bank ", stringify!($ident), " vram mode.")]
            pub fn set_bank(val: $ty) {
                unsafe { nds::$name(val.into()) }
            }
        }
    };
}

def_mod!(A::vramSetBankA(VRamTypeA));
def_mod!(B::vramSetBankB(VRamTypeB));
def_mod!(C::vramSetBankC(VRamTypeC));
def_mod!(D::vramSetBankD(VRamTypeC));

pub fn set_primary_banks(a: VRamTypeA, b: VRamTypeB, c: VRamTypeC, d: VRamTypeD) {
    unsafe {
        nds::vramSetPrimaryBanks(a.into(), b.into(), c.into(), d.into());
    }
}
