extern crate tinygfx_assets as assets;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("assets.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut font = assets::Font::load("assets/Roboto-Regular.ttf").unwrap();

    let epd_font = font.generate(
        "ROBOTO_30",
        30,
        " abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:.,%°ß↗",
        assets::FontType::Bitmap,
        "::tinygfx",
    );
    f.write_all(epd_font.as_bytes()).unwrap();
}
