pub mod loop_window;
pub mod vsync;

use crate::{
    context::{driver::Driver, phases, Context},
    gfx::GfxContext,
    looper::vsync::TargetFrameInterval,
};
use std::{num::NonZeroU32, time::Instant};
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
        mut driver: Option<Box<dyn Driver>>,
    ) -> Result<Self, LooperCreationError> {
        let gfx_ctx = GfxContext::new(window).await?;
        let ctx = Context::new(gfx_ctx);

        {
            let physical_size = window.inner_size();
            // TODO: update scale factor
            // let mut screen_mgr = ctx.screen_mgr_mut();
            // screen_mgr.update_scale_factor(scale_factor, physical_size);
            ctx.gfx_ctx().resize(physical_size);
        }

        driver.as_mut().map(|driver| driver.on_init(&ctx, window));

        Ok(Self { ctx, driver })
    }

    pub fn run(
        mut self,
        event_loop: EventLoop<()>,
        window: &'window Window,
        looper_mode: LooperMode,
        target_fps: TargetFps,
    ) -> Result<(), LooperError> {
        // TODO: perform pre-init
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

        event_loop.run(move |event, target| match event {
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

                if looper_mode == LooperMode::Poll
                    && now - last_frame_time < target_frame_interval.interval()
                {
                    return;
                }

                last_frame_time = now;

                // TODO: update time manager
                // {
                //     let mut time_mgr = self.ctx.time_mgr_mut();
                //     time_mgr.update();
                // }

                // {
                //     let mut input_mgr = self.ctx.input_mgr_mut();
                //     input_mgr.poll();
                // }

                phases::update::update(&window, &self.ctx, &mut self.driver);
                phases::late_update::late_update(&window, &self.ctx, &mut self.driver);
                // self.ctx.event_mgr().dispatch(&event_types::Update);

                // make_ui_scaler_dirty.run_now(&self.ctx.world());
                // update_ui_scaler.run_now(&self.ctx.world());
                // update_ui_element.run_now(&self.ctx.world());
                // update_ui_raycast_grid.run_now(&self.ctx.world());

                // self.ctx.ui_event_mgr_mut().handle_mouse_move();

                // {
                //     let world = self.ctx.world();
                //     let mut object_mgr = self.ctx.object_mgr_mut();
                //     let object_hierarchy = object_mgr.object_hierarchy_mut();

                //     object_hierarchy.copy_dirty_to_current_frame();

                //     let transforms = world.read_component::<Transform>();
                //     object_hierarchy.update_object_matrices(|entity| transforms.get(entity));
                // }

                // self.ctx.event_mgr().dispatch(&event_types::LateUpdate);

                phases::render::render(&window, &self.ctx, &mut self.driver);
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
                // TODO: handle keyboard input
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
                // TODO: handle resize event
                // self.ctx.screen_mgr_mut().update_size(inner_size);

                if inner_size.width == 0 || inner_size.height == 0 {
                    window_too_small = true;
                    return;
                } else {
                    window_too_small = false;
                }

                self.ctx.gfx_ctx().device.poll(MaintainBase::Wait);
                self.ctx.gfx_ctx().resize(inner_size);
                // self.ctx.render_mgr_mut().resize(inner_size);

                if looper_mode == LooperMode::Wait && !window_occluded {
                    window.request_redraw();
                }

                return;
            }
            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                window_id: id,
            } if id == window_id => {
                target_frame_interval.update_window(window);

                let inner_size = window.inner_size();
                // TODO: update scale factor
                // self.ctx
                //     .screen_mgr_mut()
                //     .update_scale_factor(scale_factor, *new_inner_size);

                if inner_size.width == 0 || inner_size.height == 0 {
                    window_too_small = true;
                    return;
                } else {
                    window_too_small = false;
                }

                self.ctx.gfx_ctx().resize(inner_size);
                // self.ctx.render_mgr_mut().resize(inner_size);

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
                    driver.on_finish(&self.ctx, window);
                }

                return;
            }
            _ => return,
        })?;

        Ok(())
    }
}
