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

fn main() {
    let file = "data/images/TESTRAMP.LBM.json";
    let text = std::fs::read_to_string(file).unwrap();

    let pattern = Regex::new(r"([a-z]*):").unwrap();

    let text = pattern
        .replace_all(&text, "\"$1\":".to_string())
        .replace("'", "\"");
    std::fs::write("output.json", &text).unwrap();

    let data: Result<CanvasCycleImage, serde_json::Error> = serde_json::from_str(&text);
    let data = data.unwrap();
}
