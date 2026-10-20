use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver},
};

#[cfg(windows)]
use crate::utils::WallpaperOptions;
use macroquad::{miniquad::window::screen_size, prelude::*};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[path = "../utils.rs"]
pub mod utils;

#[cfg(windows)]
#[path = "../windows.rs"]
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
#[derive(Serialize, Deserialize, Clone, Copy)]
struct Cycle {
    reverse: u8,
    rate: u16,
    low: u8,
    high: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CanvasCycleImage {
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

pub fn window_conf() -> Conf {
    Conf {
        window_title: "my cool canvascycler :3".to_string(),
        window_width: 640,
        window_height: 480,
        high_dpi: true,
        ..Default::default()
    }
}

pub fn parse_file_data(text: &str) -> CanvasCycleImage {
    let pattern = Regex::new(r"([a-z]*):").unwrap();
    let text = pattern
        .replace_all(text, "\"$1\":".to_string())
        .replace("'", "\"");
    let data: Result<CanvasCycleImage, serde_json::Error> = serde_json::from_str(&text);
    data.unwrap()
}

fn start_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            let mut buf = String::new();
            stdin.read_line(&mut buf).unwrap();
            let r = tx.send(buf);
            if let Err(e) = r {
                println!("{e:?}");
            }
        }
    });
    rx
}

fn read_image_data(path: &Path) -> CanvasCycleImage {
    let text = std::fs::read_to_string(path);

    let text = text.unwrap();
    parse_file_data(&text)
}

type ChangingPixels = HashMap<ChangingPixel, Vec<usize>>;

fn load_new_image(image_data: &mut [u8], data: &CanvasCycleImage) -> ChangingPixels {
    // group all pixels that need to same with the same color together
    let mut changing_pixels: ChangingPixels = HashMap::new();
    'pixel_loop: for (pixel_index, pixel) in data.pixels.iter().enumerate() {
        for (cycle_index, cycle) in data.cycles.iter().enumerate() {
            if *pixel >= cycle.low && *pixel <= cycle.high {
                if cycle.high == cycle.low || cycle.rate == 0 {
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

    #[allow(clippy::identity_op)]
    for (index, pixel) in data.pixels.iter().enumerate() {
        let c = &data.colors[*pixel as usize];
        image_data[index * 4 + 0] = c.r;
        image_data[index * 4 + 1] = c.g;
        image_data[index * 4 + 2] = c.b;
        image_data[index * 4 + 3] = 255; // alpha
    }
    changing_pixels
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut args = std::env::args().skip(1);
    let arg = args.next();
    let path = if let Some(arg) = arg {
        PathBuf::from(arg)
    } else {
        PathBuf::from("data/images/TESTRAMP.LBM.json")
    };
    let stdin_rx = start_stdin_channel();
    let mut data = read_image_data(&path);

    #[allow(unused_variables)]
    let wallpaper = args
        .next()
        .is_some_and(|a| a.as_str() == "-w" || a.as_str() == "--wallpaper");

    #[cfg(windows)]
    let as_wallpaper: Option<WallpaperOptions> = if wallpaper {
        let monitors = windows::get_monitors();
        let monitor_index: usize = args.next().and_then(|a| a.parse().ok()).unwrap_or_default();
        let monitor = monitors[monitor_index];

        Some(WallpaperOptions {
            left: monitor.left,
            top: monitor.top,
            right: monitor.right,
            bottom: monitor.bottom,
        })
    } else {
        None
    };
    let tray = args.next().is_some_and(|a| a.as_str() == "--tray");

    #[allow(unused_variables)]
    // List of all images in the directory of the loaded image.
    // Only loaded for the primary window (i.e. the one with --tray).
    let image_list = if tray {
        let parent = path.parent().unwrap();
        let mut entries = Vec::new();
        for item in std::fs::read_dir(parent).unwrap().flatten() {
            if item.file_type().unwrap().is_dir() {
                continue;
            }
            let name = item.file_name();
            let name = name.to_string_lossy();
            let Some((name, _ext)) = name.split_once(".") else {
                continue;
            };
            let mut name = name.to_string();
            name.push('\x00');
            entries.push((name, item.path()));
        }
        entries
    } else {
        Vec::new()
    };

    #[cfg(windows)]
    let hwnd;
    #[cfg(windows)]
    {
        // hwnd = windows::find_miniquad_window();

        use macroquad::miniquad::window::get_window_handle;
        hwnd = get_window_handle().unwrap() as _;
        windows::set_window_scaling();
        if tray {
            windows::set_tray(hwnd);
        }

        if let Some(wallpaper) = &as_wallpaper {
            windows::style_window_as_wallpaper(hwnd, wallpaper);
        }
    }
    let mut image_data = vec![0; data.width as usize * data.height as usize * 4];
    let mut changing_pixels = load_new_image(&mut image_data, &data);
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

    // main loop
    loop {
        #[cfg(windows)]
        if as_wallpaper.is_some() {
            let mouse = mouse_delta_position();
            if mouse != Vec2::ZERO {
                let x: u32 = mouse.x.to_bits();
                let _y: u32 = mouse.y.to_bits();
                if x == 3148524160 || x == 3137898496 {
                    windows::show_context_menu(hwnd, &image_list);
                } else if [1001040512, 3137898496, 3142372608, 990414848, 3205007497].contains(&x) {
                    // known but unhandled message ids
                } else {
                    // unknown message ids
                    // println!("unknown message id received: {x},{y}");
                }
            }
        }

        // check stdin
        if let Ok(value) = stdin_rx.try_recv() {
            match value.trim() {
                "exit" => {
                    break;
                }
                s if s.starts_with("change-wallpaper") => {
                    let new_file = &s["change-wallpaper".len()..];
                    data = read_image_data(&PathBuf::from(new_file));
                    changing_pixels = load_new_image(&mut image.bytes, &data);
                    texture.update(&image);
                }
                _ => {
                    eprintln!("Unknown command.")
                }
            }
        }

        let time = get_time();

        // only change the pixels that change
        for (changing_pixel, indices) in changing_pixels.iter() {
            let cycle = &data.cycles[changing_pixel.cycle_index as usize];
            let cycle_length = cycle.high - cycle.low;

            let interval_rate_time = interval_time / (cycle.rate as f64 / 2500.0);
            let last_time = interval_rate_time * (time / interval_rate_time).floor();
            let cycle_offset = ((time / interval_rate_time).floor() % cycle_length as f64) as u8;
            let transition_amt = (time - last_time) / interval_rate_time;

            let offset = if cycle.reverse == 0 {
                cycle_length - changing_pixel.offset + cycle_offset
            } else {
                changing_pixel.offset + cycle_offset
            };

            let mut offset_clamped = offset % cycle_length;
            let mut next_offset_clamped = (offset + 1) % cycle_length;
            if cycle.reverse == 0 {
                offset_clamped = cycle_length - offset_clamped;
                next_offset_clamped = cycle_length - next_offset_clamped;
            }

            let color_index = cycle.low + offset_clamped;
            let color = &data.colors[color_index as usize];

            let next_color_index = cycle.low + next_offset_clamped;
            let next_color = &data.colors[next_color_index as usize];

            let mixed_color = PixelRgb::mix(color, next_color, transition_amt);

            #[expect(clippy::identity_op)]
            for pixel_index in indices {
                let image_data_index = *pixel_index * 4;
                image.bytes[image_data_index + 0] = mixed_color.r;
                image.bytes[image_data_index + 1] = mixed_color.g;
                image.bytes[image_data_index + 2] = mixed_color.b;
            }
        }
        texture.update(&image);

        #[allow(unused_mut)]
        let (mut width, mut height) = screen_size();
        #[cfg(not(windows))]
        {
            // windows handles scaling differently so this isnt needed
            let scale = screen_dpi_scale();
            width /= scale;
            height /= scale;
        }

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
