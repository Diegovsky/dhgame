#![no_main]
#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use core::error::Error;
use core::ffi::*;
use glam::FloatExt;
// pub use glam::U8Vec2 as vec2;
pub use glam::Vec2 as vec2;
#[allow(unused_imports)]
use libnds::sys::{arm9_bindings as nds, eprintln, println};
use libnds::{
    Gfx, Keys, OAM, SpriteColorFormat, SpriteConfig, SpriteMapping, SpriteSize, background,
    resources,
    texture::{Palette, PaletteType, Texture},
    video::{self, VRamTypeA, VRamTypeB, VRamTypeC, VRamTypeD},
};

#[unsafe(no_mangle)]
extern "C" fn main() -> c_int {
    resources::nitrofs_init();
    unsafe {
        nds::consoleDebugInit(nds::DebugDevice_NOCASH);
    }
    match app() {
        Ok(()) => return 0,
        Err(e) => {
            unsafe {
                nds::consoleDemoInit();
            }
            println!("Error: {e}");
            loop {}
        }
    }
}

struct Sprite {
    gfx: Gfx,
    oam: OAM,
    id: u8,
}

impl Sprite {
    fn new(size: SpriteSize, format: SpriteColorFormat, oam: OAM, id: u8) -> Self {
        let gfx = oam.allocate_gfx(size, format);
        oam.set_sprite(id, &gfx, &SpriteConfig {
            ..Default::default()
        });
        Self { gfx, oam, id }
    }
    fn set_texture(&self, texture: &Texture) {
        self.gfx.set_texture(&texture.img);
    }
    fn set_pos(&self, x: u8, y: u8) {
        self.oam.set_sprite_pos(self.id, x, y);
    }
}

struct Player {
    sprite: Sprite,
    airborne: bool,
    acc: vec2,
    pos: vec2,
    vel: vec2,
}

const TICK: f32 = 1.0 / 60.0;

impl Player {
    fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            airborne: true,
            acc: Default::default(),
            pos: Default::default(),
            vel: Default::default(),
        }
    }

    fn update(&mut self, keys: Keys, just_pressed: Keys) {
        let mut xvec = 0.0;
        let maxvelx = 50.0;

        if keys.contains(Keys::LEFT) {
            xvec = -1.0;
        } else if keys.contains(Keys::RIGHT) {
            xvec = 1.0;
        }

        if self.pos.y == 160.0 {
            self.airborne = false;
        }

        if self.airborne {
            if self.vel.y < 0.0 && !keys.contains(Keys::A) {
                self.vel.y *= 0.5;
            }
        } else {
            if just_pressed.contains(Keys::A) {
                self.airborne = true;
                eprintln!("a pressed");
                self.vel.y = -self.acc.y / 2.0;
            }
        }

        let acc = 100.0;
        // Oposite movement should be faster
        if -self.vel.x.signum() == xvec {
            self.vel.x = 10.0 * xvec;
        }

        let drag = 4.0;
        self.acc.y = 110.;
        self.acc.x = if xvec != 0.0 {
            xvec * acc
        } else {
            // Stopping motion
            -self.vel.x * drag
        };

        self.vel = self.vel.clamp_length_max(maxvelx);
        self.pos += self.vel * 10.0 * TICK;
        self.vel += self.acc * TICK;
        if self.vel.x.abs() <= 0.05 {
            self.vel.x = 0.0;
        }
        // eprintln!("{:?} {:?} {:?}", self.vel, self.acc, xvec);

        self.pos = self.pos.clamp(vec2::new(0.0, 0.0), vec2::new(230.0, 160.0));
        let [x, y] = self.pos.to_array().map(|val| val as u8);
        self.sprite.set_pos(x, y);
    }
}

fn app() -> Result<(), Box<dyn Error>> {
    let bg = Texture::load("nitro:/bg/bg.img.bin")?;
    let bg_palette = Palette::load("nitro:/bg/pal.bin")?;

    let squid = Texture::load("nitro:/Squid.img.bin")?;
    let sprite_palette = Palette::load("nitro:/pal.bin")?;
    let oam_main = OAM::main();
    let oam_sub = OAM::sub();
    video::set_main(video::Mode2D::Mode5);
    video::set_sub(video::Mode2D::Mode5);
    video::set_primary_banks(
        VRamTypeA::MainSprite0,
        VRamTypeB::MainBg0,
        VRamTypeC::SubBg,
        VRamTypeD::SubSprite,
    );
    oam_main.init(SpriteMapping::SM1D128, false);
    oam_sub.init(SpriteMapping::SM1D128, false);

    bg_palette.write(&oam_main, PaletteType::Backgrounds);
    bg_palette.write(&oam_sub, PaletteType::Backgrounds);
    sprite_palette.write(&oam_main, PaletteType::Sprites);

    let bg_gfx = oam_main.allocate_bg(
        background::Layer::L2,
        background::Type::Bmp8,
        background::Bitmap8Size::B8_256x256,
        0,
        0,
    );
    let bg_gfx_sub = oam_sub.allocate_bg(
        background::Layer::L2,
        background::Type::Bmp8,
        background::Bitmap8Size::B8_256x256,
        0,
        0,
    );
    bg_gfx.set_texture(&bg);
    bg_gfx_sub.set_texture(&bg);

    let player_sprite = Sprite::new(
        SpriteSize::S16x16,
        SpriteColorFormat::SP256Color,
        oam_main,
        0,
    );
    player_sprite.set_texture(&squid);

    let mut player = Player::new(player_sprite);

    let mut last_held = Keys::empty();
    loop {
        let keys = libnds::held_keys();
        let just_pressed = keys & !last_held;
        player.update(keys, just_pressed);

        // A | A'
        // 1 | 1 = 0
        // 1 | 0 = 1
        // 0 | 1 = 0
        // 0 | 0 = 0
        last_held = keys;

        libnds::wait_for_vblank();
        background::update();
        oam_sub.update();
        oam_main.update();
    }
}
