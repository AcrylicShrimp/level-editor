mod context;
mod gfx;
mod looper;

use looper::{
    loop_window::{LoopWindow, LoopWindowConfig},
    Looper, LooperMode, TargetFps,
};
use pollster::FutureExt;

fn main() {
    let window = LoopWindow::new(LoopWindowConfig {
        title: "Level Editor".to_owned(),
        width: 800,
        height: 600,
        resizable: true,
    })
    .unwrap();
    let (event_loop, window) = window.into();

    let looper = Looper::new(&window).block_on().unwrap();
    looper
        .run(event_loop, &window, LooperMode::Poll, TargetFps::VSync)
        .unwrap();
}
