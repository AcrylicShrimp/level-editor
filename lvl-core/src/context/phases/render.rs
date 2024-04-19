use crate::{
    context::{driver::Driver, Context},
    gfx::{ClearMode, Frame, RenderPassTarget},
};
use wgpu::{Color, TextureView};
use winit::window::Window;

pub fn render(window: &Window, ctx: &Context, driver: &mut Option<Box<dyn Driver>>) {
    if let Some(driver) = driver {
        driver.on_before_render(&ctx, window);
    }

    // TODO: render frame here

    // update_camera_transform_buffer_system.run_now(&self.ctx.world());
    // render_system.run_now(&self.ctx.world());

    let surface_texture = ctx.gfx_ctx().obtain_surface_view().unwrap();
    let surface_texture_view = surface_texture.texture.create_view(&Default::default());

    let mut frame = ctx.gfx_ctx().begin_frame();
    {
        render_pass_stage_opaque(ctx, &surface_texture_view, &mut frame);
        render_pass_stage_ui(ctx, &surface_texture_view, &mut frame);
    }
    ctx.gfx_ctx().end_frame(frame);

    window.pre_present_notify();
    surface_texture.present();

    if let Some(driver) = driver {
        driver.on_after_render(&ctx, window);
    }
}

fn render_pass_stage_opaque(ctx: &Context, surface_texture_view: &TextureView, frame: &mut Frame) {
    let render_pass = frame.begin_render_pass(
        ClearMode::All {
            color: Color::BLACK,
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

fn render_pass_stage_ui(ctx: &Context, surface_texture_view: &TextureView, frame: &mut Frame) {
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
