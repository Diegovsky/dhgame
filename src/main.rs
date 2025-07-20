#![no_main]
#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use core::error::Error;
use core::ffi::*;
// pub use glam::U8Vec2 as vec2;
pub use glam::Vec2 as vec2;
#[allow(unused_imports)]
use libnds::sys::{arm9_bindings as nds, eprintln, println};
use libnds::{
    Gfx, Keys, OAM, SpriteColorFormat, SpriteConfig, SpriteEntry, SpriteMapping, SpriteSize,
    background::{self as bg, BackgroundPtr},
    fill_slice, fill_slice_u8, resources,
    texture::{Palette, PaletteType, Texture},
    video::{self, SCREEN_HEIGHT, SCREEN_WIDTH, VRamTypeA, VRamTypeB, VRamTypeC, VRamTypeD},
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
    fn entry(&mut self) -> &'static mut SpriteEntry {
        &mut self.oam.sprites()[self.id as usize]
    }
    const fn size(&self) -> SpriteSize {
        self.gfx.size()
    }
}

struct Player {
    edata: EntityData,
    airborne: bool,
}

const TICK: f32 = 1.0 / 60.0;

impl Player {
    fn new(sprite: Sprite) -> Self {
        Self {
            edata: EntityData::new(sprite, Default::default()),
            airborne: true,
        }
    }
}

impl Entity for Player {
    fn child_mut(&mut self) -> Option<&mut dyn Entity> {
        Some(&mut self.edata)
    }
    fn update(
        &mut self,
        update_data @ UpdateData {
            keys, just_pressed, ..
        }: &UpdateData,
    ) {
        let mut xvec = 0.0;
        let maxvelx = 50.0;
        let screen_end = vec2::new(SCREEN_WIDTH as f32 - 16.0, SCREEN_HEIGHT as f32 - 16.0);

        if keys.contains(Keys::LEFT) {
            xvec = -1.0;
        } else if keys.contains(Keys::RIGHT) {
            xvec = 1.0;
        }

        if self.edata.pos.y >= screen_end.y {
            self.airborne = false;
        }

        if self.airborne {
            if self.edata.vel.y < 0.0 && !keys.contains(Keys::A) {
                self.edata.vel.y *= 0.5;
            }
        } else {
            if just_pressed.contains(Keys::A) {
                self.airborne = true;
                eprintln!("a pressed");
                self.edata.vel.y = -self.edata.acc.y / 2.0;
            }
        }

        let acc = 100.0;
        // Oposite movement should be faster
        if -self.edata.vel.x.signum() == xvec {
            self.edata.vel.x = 10.0 * xvec;
        }

        let drag = 4.0;
        self.edata.acc.y = 110.;
        self.edata.acc.x = if xvec != 0.0 {
            xvec * acc
        } else {
            // Stopping motion
            -self.edata.vel.x * drag
        };

        self.edata.vel = self.edata.vel.clamp_length_max(maxvelx);
        self.edata.update(update_data);
        if self.edata.vel.x.abs() <= 0.05 {
            self.edata.vel.x = 0.0;
        }
        // eprintln!("{:?} {:?} {:?}", self.edata.vel, self.edata.acc, xvec);

        self.edata.pos = self.edata.pos.clamp(vec2::new(0.0, 0.0), screen_end);
    }
}

struct EntityData {
    sprite: Sprite,
    pos: vec2,
    vel: vec2,
    acc: vec2,
}

impl EntityData {
    fn new(sprite: Sprite, pos: vec2) -> Self {
        Self {
            sprite,
            pos,
            vel: vec2::ZERO,
            acc: vec2::ZERO,
        }
    }
}

impl Entity for EntityData {
    fn update(&mut self, update_data: &UpdateData) {
        self.pos += self.vel * 10.0 * TICK;
        self.vel += self.acc * TICK;
    }

    fn data_mut(&mut self) -> &mut EntityData {
        self
    }
}

#[derive(Clone, Copy)]
struct UpdateData {
    keys: Keys,
    just_pressed: Keys,
    camera: f32,
}

trait Entity {
    fn update(&mut self, update_data: &UpdateData);

    fn data_mut(&mut self) -> &mut EntityData {
        self.child_mut().unwrap().data_mut()
    }

    fn child_mut(&mut self) -> Option<&mut dyn Entity> {
        None
    }
}

fn update(entity: &mut dyn Entity, update_data: &UpdateData) {
    if let Some(child) = entity.child_mut() {
        // update(child, update_data);
    }
    entity.update(update_data);
    let data = entity.data_mut();
    let start = data.pos;
    let end = start
        + vec2::new(
            data.sprite.size().height() as _,
            data.sprite.size().width() as _,
        );
    let hidden = end.cmple(vec2::ZERO).any()
        || start
            .cmpge(vec2::new(SCREEN_WIDTH as _, SCREEN_HEIGHT as _))
            .any();
    data.sprite.entry().set_is_hidden(hidden);

    let [x, y] = data.pos.to_array().map(|val| val as u8);
    data.sprite.set_pos(x, y);
}

fn app() -> Result<(), Box<dyn Error>> {
    let bg = Texture::load("nitro:/bg/bg.img.bin")?;
    let bg_palette = Palette::load("nitro:/bg/pal.bin")?;

    let squid = Texture::load("nitro:/Squid.img.bin")?;
    let platform = Texture::load("nitro:/Platform.img.bin")?;
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
        bg::Layer::L2,
        bg::Type::Bmp8,
        bg::Bitmap8Size::B8_256x256,
        0,
        0,
    );
    let bg_gfx_sub = oam_sub.allocate_bg(
        bg::Layer::L2,
        bg::Type::Bmp8,
        bg::Bitmap8Size::B8_256x256,
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

    let plat_sprite = Sprite::new(
        SpriteSize::S32x16,
        SpriteColorFormat::SP256Color,
        oam_main,
        1,
    );
    plat_sprite.set_texture(&platform);
    let mut platform = EntityData::new(plat_sprite, Default::default());

    let mut player = Player::new(player_sprite);

    let mut last_held = Keys::empty();
    let mut camera = 0.0;
    platform.vel.x = 100.0;
    loop {
        let keys = libnds::held_keys();
        let just_pressed = keys & !last_held;
        last_held = keys;
        if keys.contains(Keys::UP) {
            camera += 1.0;
        } else if keys.contains(Keys::DOWN) {
            camera -= 1.0;
        }
        let update_data = UpdateData {
            keys,
            just_pressed,
            camera,
        };

        let entities: &mut [&mut dyn Entity] = &mut [&mut player, &mut platform];
        for entity in entities {
            update(*entity, &update_data);
        }

        libnds::wait_for_vblank();
        bg::update();
        oam_sub.update();
        oam_main.update();
    }
}
