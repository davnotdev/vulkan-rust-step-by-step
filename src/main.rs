// I like explicitly showing where numbers come from.
#![allow(clippy::erasing_op)]
#![allow(clippy::identity_op)]

mod app;
mod vulkan;

use app::App;

fn main() {
    let (app, eloop) = App::create();
    app.run(eloop).unwrap();
}
