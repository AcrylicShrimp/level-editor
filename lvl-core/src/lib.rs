pub mod context;
pub mod gfx;
pub mod looper;
pub mod perf;
pub mod resource;
pub mod scene;

use context::driver::Driver;
use looper::{
    loop_window::{LoopWindow, LoopWindowConfig},
    Looper, LooperMode, TargetFps,
};
use pollster::FutureExt;

pub fn launch_core(
    window_config: LoopWindowConfig,
    vsync: bool,
    looper_mode: LooperMode,
    target_fps: TargetFps,
    driver: Option<Box<dyn Driver>>,
) {
    let window = LoopWindow::new(window_config).unwrap();
    let (event_loop, window) = window.into();

    let looper = Looper::new(&window, vsync, driver).block_on().unwrap();
    looper
        .run(event_loop, &window, looper_mode, target_fps)
        .unwrap();
}
