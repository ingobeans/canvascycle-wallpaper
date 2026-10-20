use crate::renderer::utils::WallpaperOptions;

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

    let monitors = renderer::windows::get_monitors();

    let mut handles = Vec::new();
    for monitor in monitors {
        let wallpaper = Some(WallpaperOptions {
            left: monitor.left,
            top: monitor.top,
            right: monitor.right,
            bottom: monitor.bottom,
        });

        let data_clone = data.clone();
        let handle = std::thread::spawn(move || {
            macroquad::Window::from_config(
                renderer::window_conf(),
                renderer::show(data_clone, wallpaper),
            );
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
