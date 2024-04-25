use super::common::{get_all_cameras, CameraObject};
use crate::{
    context::{driver::Driver, Context},
    gfx::{ClearMode, Frame, RenderPassTarget},
    scene::{components::CameraClearMode, Scene},
};
use wgpu::{Color, TextureView};
use winit::window::Window;

pub fn render(
    window: &Window,
    ctx: &Context,
    scene: &mut Scene,
    driver: &mut Option<Box<dyn Driver>>,
) {
    if let Some(driver) = driver {
        driver.on_before_render(&ctx, window, scene);
    }

    // TODO: render frame here

    // update_camera_transform_buffer_system.run_now(&self.ctx.world());
    // render_system.run_now(&self.ctx.world());

    let surface_texture = ctx.gfx_ctx().obtain_surface_view().unwrap();
    let surface_texture_view = surface_texture.texture.create_view(&Default::default());

    let mut frame = ctx.gfx_ctx().begin_frame();

    for camera in get_all_cameras(&scene.read_only_proxy()) {
        render_pass_stage_opaque(ctx, &surface_texture_view, &mut frame, &camera);
        render_pass_stage_ui(ctx, &surface_texture_view, &mut frame, &camera);
    }

    ctx.gfx_ctx().end_frame(frame);

    window.pre_present_notify();
    surface_texture.present();

    if let Some(driver) = driver {
        driver.on_after_render(&ctx, window, scene);
    }
}

fn render_pass_stage_opaque(
    ctx: &Context,
    surface_texture_view: &TextureView,
    frame: &mut Frame,
    camera: &CameraObject,
) {
    let render_pass = frame.begin_render_pass(
        match camera.camera.clear_mode {
            CameraClearMode::All { color } => ClearMode::All {
                color: Color {
                    r: color.x as f64,
                    g: color.y as f64,
                    b: color.z as f64,
                    a: color.w as f64,
                },
                depth: 1.0,
                stencil: 0,
            },
            CameraClearMode::DepthStencilOnly => ClearMode::DepthStencilOnly {
                depth: 1.0,
                stencil: 0,
            },
            CameraClearMode::Keep => ClearMode::Keep,
        },
        &[Some(RenderPassTarget {
            view: &surface_texture_view,
            writable: true,
        })],
        None,
    );

    // TODO: draw something
}

fn render_pass_stage_ui(
    ctx: &Context,
    surface_texture_view: &TextureView,
    frame: &mut Frame,
    camera: &CameraObject,
) {
    let render_pass = frame.begin_render_pass(
        ClearMode::DepthStencilOnly {
            depth: 1.0,
            stencil: 0,
        },
        &[Some(RenderPassTarget {
            view: &surface_texture_view,
            writable: true,
        })],
        None,
    );

    // TODO: draw something
}
