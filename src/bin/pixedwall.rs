mod renderer;
fn main() {
    let mut args = std::env::args().skip(1);
    let arg = args.next();
    let text = if let Some(arg) = arg {
        std::fs::read_to_string(arg.as_str())
    } else {
        std::fs::read_to_string("data/images/TESTRAMP.LBM.json")
    };
    let text = text.unwrap();
    let data = renderer::parse_file_data(&text);

    let handle = std::thread::spawn(move || {
        macroquad::Window::from_config(renderer::window_conf(), renderer::show(data.clone(), None));
    });
    handle.join().unwrap();
}
