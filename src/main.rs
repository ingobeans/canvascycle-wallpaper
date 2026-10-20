#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use macroquad::{miniquad::window::screen_size, prelude::*};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[cfg(windows)]
mod windows;

#[derive(Serialize, Deserialize, Clone, Copy)]
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
    fn mix(a: &PixelRgb, b: &PixelRgb, amt: f64) -> Self {
        Self {
            r: (a.r as f64 + (b.r as f64 - a.r as f64) * amt) as u8,
            g: (a.g as f64 + (b.g as f64 - a.g as f64) * amt) as u8,
            b: (a.b as f64 + (b.b as f64 - a.b as f64) * amt) as u8,
        }
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
    let mut args = std::env::args().skip(1);
    let arg = args.next();
    let text = if let Some(arg) = arg {
        std::fs::read_to_string(arg.as_str())
    } else {
        std::fs::read_to_string("data/images/TESTRAMP.LBM.json")
    };
    let text = text.unwrap();

    #[cfg(windows)]
    {
        let wallpaper = args
            .next()
            .is_some_and(|a| a.as_str() == "-w" || a.as_str() == "--wallpaper");
        if wallpaper && let Some(hwnd) = windows::find_miniquad_window() {
            println!("hwnd found!");
            windows::style_window_as_wallpaper(hwnd);
        }
    }

    let pattern = Regex::new(r"([a-z]*):").unwrap();

    let text = pattern
        .replace_all(&text, "\"$1\":".to_string())
        .replace("'", "\"");
    std::fs::write("output.json", &text).unwrap();

    let data: Result<CanvasCycleImage, serde_json::Error> = serde_json::from_str(&text);
    let data = data.unwrap();

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

    let interval_time = 0.1;
    let integer_scaling = false;

    let mut fullscreen = false;
    loop {
        let time = get_time();

        // only change the pixels that change
        for (changing_pixel, indices) in changing_pixels.iter() {
            let cycle = &data.cycles[changing_pixel.cycle_index as usize];
            let cycle_length = cycle.high - cycle.low;

            let mixed_color;
            if cycle.rate != 0 {
                let interval_rate_time = interval_time / (cycle.rate as f64 / 2500.0);
                let last_time = interval_rate_time * (time / interval_rate_time).floor();
                let cycle_offset =
                    ((time / interval_rate_time).floor() % cycle_length as f64) as u8;
                let transition_amt = (time - last_time) / interval_rate_time;

                let offset = cycle_length - changing_pixel.offset + cycle_offset;

                let offset_clamped = cycle_length - offset % cycle_length;
                let next_offset_clamped = cycle_length - (offset + 1) % cycle_length;

                let color_index = cycle.low + offset_clamped;
                let color = &data.colors[color_index as usize];

                let next_color_index = cycle.low + next_offset_clamped;
                let next_color = &data.colors[next_color_index as usize];

                mixed_color = PixelRgb::mix(color, next_color, transition_amt);
            } else {
                mixed_color = data.colors[(changing_pixel.offset + cycle.low) as usize];
            }

            #[expect(clippy::identity_op)]
            for pixel_index in indices {
                let image_data_index = *pixel_index * 4;
                image.bytes[image_data_index + 0] = mixed_color.r;
                image.bytes[image_data_index + 1] = mixed_color.g;
                image.bytes[image_data_index + 2] = mixed_color.b;
            }
        }
        texture.update(&image);

        let (width, height) = screen_size();
        let mut scale_factor = (width / data.width as f32).min(height / data.height as f32);
        if integer_scaling {
            scale_factor = scale_factor.floor();
        }
        let dest_width = data.width as f32 * scale_factor;
        let dest_height = data.height as f32 * scale_factor;
        let x_offset = (width - dest_width) / 2.0;
        let y_offset = (height - dest_height) / 2.0;
        draw_texture_ex(
            &texture,
            x_offset,
            y_offset,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(dest_width, dest_height)),
                ..Default::default()
            },
        );
        if is_key_pressed(KeyCode::F11) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        next_frame().await;
    }
}
