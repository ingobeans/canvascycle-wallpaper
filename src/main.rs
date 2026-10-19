use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use macroquad::{miniquad::window::screen_size, prelude::*};
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
#[derive(PartialEq, Eq)]
struct ChangingPixel {
    cycle_index: u8,
    offset: u8,
}
impl Hash for ChangingPixel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u16(self.cycle_index as u16 | ((self.offset as u16) << 8));
    }
}

fn window() -> Conf {
    Conf {
        window_title: "my cool canvascycler :3".to_string(),
        window_width: 640,
        window_height: 480,
        ..Default::default()
    }
}

#[macroquad::main(window)]
async fn main() {
    let arg = std::env::args().nth(1);
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

    let mut cycle_offsets = vec![0; data.cycles.len()];

    // group all pixels that need to same with the same color together
    let mut changing_pixels: HashMap<ChangingPixel, Vec<usize>> = HashMap::new();
    'pixel_loop: for (pixel_index, pixel) in data.pixels.iter().enumerate() {
        for (cycle_index, cycle) in data.cycles.iter().enumerate() {
            if *pixel >= cycle.low && *pixel <= cycle.high {
                if cycle.high == cycle.low {
                    continue;
                }
                let offset = *pixel - cycle.low;
                let changing_pixel = ChangingPixel {
                    cycle_index: cycle_index as _,
                    offset,
                };
                match changing_pixels.entry(changing_pixel) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(pixel_index);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(vec![pixel_index]);
                    }
                }
                continue 'pixel_loop;
            }
        }
    }

    let mut image_data = Vec::with_capacity(data.width as usize * data.height as usize * 4);
    for pixel in &data.pixels {
        let c = &data.colors[*pixel as usize];
        image_data.push(c.r);
        image_data.push(c.g);
        image_data.push(c.b);
        image_data.push(255); // alpha
    }
    let mut image = Image {
        width: data.width,
        height: data.height,
        bytes: image_data,
    };
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let mut frame_counter = 0;
    loop {
        // update image when frame counter is 0
        if frame_counter == 0 {
            for (index, cycle_offset) in cycle_offsets.iter_mut().enumerate() {
                let cycle_length = data.cycles[index].high - data.cycles[index].low;
                if cycle_length == 0 {
                    continue;
                }

                *cycle_offset = (*cycle_offset + 1) % cycle_length;
            }

            // only change the pixels that change
            for (changing_pixel, indices) in changing_pixels.iter() {
                let cycle = &data.cycles[changing_pixel.cycle_index as usize];
                let cycle_length = cycle.high - cycle.low;
                let cycle_offset = cycle_offsets[changing_pixel.cycle_index as usize];

                let offset = cycle_length - changing_pixel.offset + cycle_offset;
                let offset_clamped = cycle_length - offset % cycle_length;

                let color_index = cycle.low + offset_clamped;
                let color = &data.colors[color_index as usize];

                #[expect(clippy::identity_op)]
                for pixel_index in indices {
                    let image_data_index = *pixel_index * 4;
                    image.bytes[image_data_index + 0] = color.r;
                    image.bytes[image_data_index + 1] = color.g;
                    image.bytes[image_data_index + 2] = color.b;
                }
            }
            texture.update(&image);

            frame_counter = 10;
        } else {
            frame_counter -= 1;
        }

        let (width, height) = screen_size();
        let scale_factor = (width / data.width as f32).min(height / data.height as f32);
        // .floor();
        let dest_width = data.width as f32 * scale_factor;
        let dest_height = data.height as f32 * scale_factor;
        let x_offset = (width - dest_width) / 2.0;
        draw_texture_ex(
            &texture,
            x_offset,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(dest_width, dest_height)),
                ..Default::default()
            },
        );
        next_frame().await;
    }
}
