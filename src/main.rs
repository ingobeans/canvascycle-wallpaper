use regex::Regex;

fn main() {
    let file = "data/images/TESTRAMP.LBM.json";
    let text = std::fs::read_to_string(file).unwrap();

    let pattern = Regex::new(r"([a-z]*):").unwrap();

    let text = pattern
        .replace_all(&text, "\"$1\":".to_string())
        .replace("'", "\"");
    std::fs::write("output.json", text).unwrap();
}
