use std::mem::MaybeUninit;

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
    let arg = std::env::args().skip(1).next();
    let text = if let Some(arg) = arg {
        std::fs::read_to_string(arg.as_str())
    } else {
        std::fs::read_to_string("data/images/TESTRAMP.LBM.json")
    };
    let text = text.unwrap();

    let pattern = Regex::new(r"([a-z]*):").unwrap();

    let text = pattern
        .replace_all(&text, "\"$1\":".to_string())
        .replace("'", "\"");
    std::fs::write("output.json", &text).unwrap();

    let data: Result<CanvasCycleImage, serde_json::Error> = serde_json::from_str(&text);
    let data = data.unwrap();

    #[allow(invalid_value)]
    // should be fine since frame_counter will always be zero on loop's first iteration,
    // which in turn would make sure it has an assigned value
    let mut texture = unsafe { MaybeUninit::uninit().assume_init() };

    let mut color_index = 0;
    let mut frame_counter = 0;
    loop {
        if frame_counter == 0 {
            color_index += 1;
            frame_counter = 10;
            let mut image_data = Vec::with_capacity(data.width as usize * data.height as usize * 4);
            for pixel in &data.pixels {
                let mut pixel = *pixel;
                for cycle in &data.cycles {
                    if cycle.high == cycle.low {
                        continue;
                    }
                    if pixel >= cycle.low && pixel <= cycle.high {
                        // this got kinda overcomplicated but works ig :thumbsup:
                        let base_offset = (cycle.high - cycle.low) - (pixel - cycle.low);
                        let offset = base_offset + color_index;
                        let clamped_offset =
                            (cycle.high - cycle.low) - offset % (cycle.high - cycle.low);
                        pixel = cycle.low + clamped_offset;
                        break;
                    }
                }
                let c = &data.colors[pixel as usize];
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
            texture = Texture2D::from_image(&image);
        } else {
            frame_counter -= 1;
        }
        draw_texture(&texture, 0.0, 0.0, WHITE);
        next_frame().await;
    }
}
