pub mod loop_window;
pub mod vsync;

use crate::{
    context::{driver::Driver, phases, Context},
    gfx::GfxContext,
    looper::vsync::TargetFrameInterval,
    perf::PerfRecorder,
    scene::Scene,
};
use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};
use thiserror::Error;
use wgpu::MaintainBase;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Error, Debug)]
pub enum LooperCreationError {
    #[error("winit external error: {0}")]
    WinitExternalError(#[from] winit::error::ExternalError),
    #[error("winit not supported error: {0}")]
    WinitNotSupportedError(#[from] winit::error::NotSupportedError),
    #[error("gfx context creation error: {0}")]
    GfxContextCreationError(#[from] crate::gfx::GfxContextCreationError),
}

#[derive(Error, Debug)]
pub enum LooperError {
    #[error("event loop error: {0}")]
    EventLoopError(#[from] winit::error::EventLoopError),
    #[error("gfx surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LooperMode {
    Poll,
    Wait,
}

impl Default for LooperMode {
    fn default() -> Self {
        Self::Poll
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetFps {
    VSync,
    MilliHertz(NonZeroU32),
    Unlimited,
}

impl Default for TargetFps {
    fn default() -> Self {
        Self::VSync
    }
}

pub struct Looper<'window> {
    ctx: Context<'window>,
    driver: Option<Box<dyn Driver>>,
}

#[derive(Debug, Clone, Hash)]
pub struct LooperConfig {
    title: String,
    resizable: bool,
    width: u32,
    height: u32,
}

impl<'window> Looper<'window> {
    pub async fn new(
        window: &'window Window,
        vsync: bool,
        msaa_sample_count: u32,
        driver: Option<Box<dyn Driver>>,
    ) -> Result<Self, LooperCreationError> {
        let physical_size = window.inner_size();
        let gfx_ctx = GfxContext::new(window, vsync, msaa_sample_count).await?;
        let ctx = Context::new(gfx_ctx, physical_size);
        Ok(Self { ctx, driver })
    }

    pub fn run(
        mut self,
        event_loop: EventLoop<()>,
        window: &'window Window,
        looper_mode: LooperMode,
        target_fps: TargetFps,
    ) -> Result<(), LooperError> {
        event_loop.set_control_flow(match looper_mode {
            LooperMode::Wait => ControlFlow::Wait,
            LooperMode::Poll => ControlFlow::Poll,
        });
        window.set_visible(true);

        let window_id = window.id();
        let mut window_occluded = false;
        let mut window_too_small = false;
        let mut target_frame_interval = TargetFrameInterval::new(
            match target_fps {
                TargetFps::VSync => None,
                TargetFps::MilliHertz(millihertz) => Some(millihertz),
                TargetFps::Unlimited => None,
            },
            window,
        );
        let mut last_frame_time = Instant::now();
        let mut scene = Scene::new(&self.ctx, window);
        let mut perf_recorder = PerfRecorder::new("main");
        let mut last_perf_report_time = Instant::now();

        self.driver
            .as_mut()
            .map(|driver| driver.on_init(&self.ctx, window, &mut scene));

        event_loop.run(|event, target| match event {
            Event::NewEvents(cause) if cause == StartCause::Poll => {
                if !window_occluded && !window_too_small {
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id: id,
            } if id == window_id => {
                let now = Instant::now();

                #[cfg(target_os = "macos")]
                if looper_mode == LooperMode::Poll
                    && now - last_frame_time < target_frame_interval.interval()
                {
                    return;
                }

                last_frame_time = now;
                self.ctx.time_mut().update();

                perf_recorder.frame_begin();

                // {
                //     let mut input_mgr = self.ctx.input_mgr_mut();
                //     input_mgr.poll();
                // }

                phases::update::update(&window, &self.ctx, &mut scene, &mut self.driver);
                perf_recorder.frame_update_end();

                phases::late_update::late_update(&window, &self.ctx, &mut scene, &mut self.driver);
                perf_recorder.frame_late_update_end();

                scene.prepare_render(&mut self.ctx.screen_size_mut());
                perf_recorder.frame_prepare_render_end();

                phases::render::render(&window, &self.ctx, &mut scene, &mut self.driver);
                perf_recorder.frame_render_end();

                if Duration::from_secs(1) <= now - last_perf_report_time {
                    println!("{}", perf_recorder.report());
                    last_perf_report_time = now;
                }

                self.ctx.input_mut().reset_current_frame_state();

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::Occluded(occluded),
                window_id: id,
            } if id == window_id => {
                window_occluded = occluded;

                if looper_mode == LooperMode::Wait && !occluded && !window_too_small {
                    window.request_redraw();
                }

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                window_id: id,
            } if id == window_id => {
                self.ctx.input_mut().handle_key_event(&event);

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorEntered { .. },
                window_id: id,
            } if id == window_id => {
                // TODO: Handle cursor entered event here.

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::CursorLeft { .. },
                window_id: id,
            } if id == window_id => {
                // TODO: Handle cursor left event here.
                return;
            }
            Event::WindowEvent {
                event: event @ WindowEvent::CursorMoved { .. },
                window_id: id,
            } if id == window_id => {
                // TODO: Handle cursor moved event here.
                return;
            }
            Event::WindowEvent {
                event: event @ WindowEvent::MouseInput { .. },
                window_id: id,
            } if id == window_id => {
                // TODO: Handle mouse input event here.
                return;
            }
            Event::WindowEvent {
                event: event @ WindowEvent::MouseWheel { .. },
                window_id: id,
            } if id == window_id => {
                // TODO: Handle mouse wheel event here.
                return;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(inner_size),
                window_id: id,
            } if id == window_id => {
                if inner_size.width == 0 || inner_size.height == 0 {
                    window_too_small = true;
                    return;
                } else {
                    window_too_small = false;
                }

                self.ctx.update_screen_size(inner_size);
                self.ctx.gfx_ctx().device.poll(MaintainBase::Wait);
                self.ctx.gfx_ctx().resize(inner_size);

                if looper_mode == LooperMode::Wait && !window_occluded {
                    window.request_redraw();
                }

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { .. },
                window_id: id,
            } if id == window_id => {
                target_frame_interval.update_window(window);

                let inner_size = window.inner_size();

                if inner_size.width == 0 || inner_size.height == 0 {
                    window_too_small = true;
                    return;
                } else {
                    window_too_small = false;
                }

                self.ctx.update_screen_size(inner_size);
                self.ctx.gfx_ctx().resize(inner_size);

                if looper_mode == LooperMode::Wait && !window_occluded {
                    window.request_redraw();
                }

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: id,
            } if id == window_id => {
                target.exit();

                if let Some(driver) = self.driver.as_mut() {
                    driver.on_finish(&self.ctx, window, &mut scene);
                }

                return;
            }
            _ => return,
        })?;

        Ok(())
    }
}
