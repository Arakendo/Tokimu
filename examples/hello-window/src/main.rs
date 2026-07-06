use tokimu::{App, WindowConfig};

fn main() {
    let app = App::new();
    let window = WindowConfig::default();

    println!(
        "Tokimu hello-window placeholder: {}x{} titled '{}' with {} phases",
        window.width,
        window.height,
        window.title,
        app.schedule.phases().len()
    );
}
