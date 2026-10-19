use macroquad::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PixelRgb {
    r: u8,
    g: u8,
    b: u8,
}
impl PixelRgb {
    #[expect(dead_code)]
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
#[derive(Serialize, Deserialize)]
struct Cycle {
    reverse: u8, // dont know how this property works
    rate: u16,
    low: u8,
    high: u8,
}

#[derive(Serialize, Deserialize)]
struct CanvasCycleImage {
    filename: String,
    width: u16,
    height: u16,
    colors: Vec<PixelRgb>,
    cycles: Vec<Cycle>,
    pixels: Vec<u8>,
}

#[macroquad::main("my cool canvascycler :3")]
async fn main() {
    let file = "data/images/TESTRAMP.LBM.json";
    let text = std::fs::read_to_string(file).unwrap();

    let pattern = Regex::new(r"([a-z]*):").unwrap();

    let text = pattern
        .replace_all(&text, "\"$1\":".to_string())
        .replace("'", "\"");
    std::fs::write("output.json", &text).unwrap();

    let data: Result<CanvasCycleImage, serde_json::Error> = serde_json::from_str(&text);
    let data = data.unwrap();

    let mut image_data = Vec::with_capacity(data.width as usize * data.height as usize * 4);
    for pixel in &data.pixels {
        let c = &data.colors[*pixel as usize];
        image_data.push(c.r);
        image_data.push(c.g);
        image_data.push(c.b);
        image_data.push(255); // alpha
    }
    let image = Image {
        width: data.width,
        height: data.height,
        bytes: image_data,
    };
    let texture = Texture2D::from_image(&image);

    loop {
        draw_texture(&texture, 0.0, 0.0, WHITE);
        next_frame().await;
    }
}
