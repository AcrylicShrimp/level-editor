use lvl_core::{
    launch_core,
    looper::{loop_window::LoopWindowConfig, LooperMode, TargetFps},
};

fn main() {
    let window_config = LoopWindowConfig {
        title: "Level Editor".to_owned(),
        width: 800,
        height: 600,
        resizable: true,
    };
    let looper_mode = LooperMode::Poll;
    let target_fps = TargetFps::VSync;

    launch_core(window_config, looper_mode, target_fps, None);
}
