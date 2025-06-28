# DH Game
Temporary codename for a Nintendo DS Game I'm developing while figuring out stuff.

## How to build and run
You will need BlocksDS installed from the Wonderful toolchain.

Also, `melonDS`, `grate` (my personal `grit` reimplementation) and `just` (command runner).

After having everything installed, do `just run`.

## Dir tree
- `src`: the game
- `vendor/libnds`: my high-ever level wrapper around `libnds`
- `vendor/libnds-sys`: my fork of `SeleDreams/libnds-sys`
- `data`: dev assets
