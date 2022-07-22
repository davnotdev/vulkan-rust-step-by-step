mod app;
mod vulkan;

use app::App;

fn main() {
    let (app, eloop) = App::create();
    app.run(eloop).unwrap();
}
