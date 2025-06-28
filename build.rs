use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn process_sprites(out: &Path, data_path: &Path) {
    let mut sprite_imgs = vec![];
    for file in data_path.read_dir().unwrap() {
        let file = file.unwrap();
        let file = PathBuf::from(file.file_name());
        if file.extension().map(|ext| ext == "png").unwrap_or(false) {
            println!("cargo:rerun-if-changed={}", file.display());
            sprite_imgs.push(data_path.join(file));
        }
    }
    let mut cmd = Command::new("grate");
    let result = cmd
        .args(["-a", "#FF00FF", "-f", "tiled", "-p", "P256", "-o"])
        .arg(out)
        .args(sprite_imgs)
        .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !result.success() {
        panic!()
    }
}

fn process_bgs(out: &Path, data_path: &Path) {
    let mut sprite_imgs = vec![];
    if !out.exists() {
        std::fs::create_dir(&out).unwrap();
    }
    for file in data_path.read_dir().unwrap() {
        let file = file.unwrap();
        let file = PathBuf::from(file.file_name());
        if file.extension().map(|ext| ext == "png").unwrap_or(false) {
            println!("cargo:rerun-if-changed={}", file.display());
            sprite_imgs.push(data_path.join(file));
        }
    }
    let mut cmd = Command::new("grate");
    let result = cmd
        .args(["-a", "#FF00FF", "-f", "bitmap", "-p", "P256", "-o"])
        .arg(out)
        .args(sprite_imgs)
        .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !result.success() {
        panic!()
    }
}
fn main() {
    let out = Path::new("romfs");
    if !out.exists() {
        std::fs::create_dir(out).unwrap();
    }
    let data_path = Path::new("data/");
    process_sprites(out, data_path);
    process_bgs(&out.join("bg"), &data_path.join("bg"));
    let out = out.canonicalize().unwrap();
    let out = out.as_path();
    println!("cargo::rustc-env=BUILD_DIR={}", out.display());
}
