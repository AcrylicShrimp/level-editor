use thiserror::Error;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

#[derive(Error, Debug)]
pub enum LoopWindowCreationError {
    #[error("failed to create event loop: {0}")]
    EventLoopCreationFailed(#[from] winit::error::EventLoopError),
    #[error("failed to create window: {0}")]
    WindowCreationFailed(#[from] winit::error::OsError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoopWindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

#[derive(Debug)]
pub struct LoopWindow {
    event_loop: EventLoop<()>,
    window: Window,
}

impl LoopWindow {
    pub fn new(config: LoopWindowConfig) -> Result<Self, LoopWindowCreationError> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_visible(false)
            .with_title(config.title)
            .with_resizable(config.resizable)
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .build(&event_loop)?;

        Ok(Self { event_loop, window })
    }

    pub fn into(self) -> (EventLoop<()>, Window) {
        (self.event_loop, self.window)
    }
}
