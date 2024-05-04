mod render_command;
mod render_static_mesh_renderer;

use self::render_static_mesh_renderer::build_render_command_static_mesh_renderer;
use super::common::get_all_cameras;
use crate::{
    context::{driver::Driver, Context},
    gfx::{ClearMode, Frame, RenderPassTarget},
    scene::{
        components::{Camera, CameraClearMode, StaticMeshRenderer},
        ObjectId, Scene, SceneProxy,
    },
};
use lvl_math::{Vec3, Vec4};
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

    scene.with_proxy(|proxy| {
        for camera_id in get_all_cameras(proxy) {
            render_pass_stage_opaque(ctx, camera_id, &surface_texture_view, &mut frame, proxy);
            render_pass_stage_ui(ctx, camera_id, &surface_texture_view, &mut frame, proxy);
        }
    });

    ctx.gfx_ctx().end_frame(frame);

    window.pre_present_notify();
    surface_texture.present();

    if let Some(driver) = driver {
        driver.on_after_render(&ctx, window, scene);
    }
}

fn render_pass_stage_opaque(
    ctx: &Context,
    camera_id: ObjectId,
    surface_texture_view: &TextureView,
    frame: &mut Frame,
    scene: &mut SceneProxy,
) {
    let camera = scene
        .find_object_by_id(camera_id)
        .unwrap()
        .find_component_by_type::<Camera>()
        .unwrap();

    let camera_world_pos =
        scene.transform_matrix(camera_id).unwrap() * Vec4::new(0.0, 0.0, 0.0, 1.0);
    let mut commands = Vec::new();

    if let Some(ids) = scene.find_object_ids_by_component_type::<StaticMeshRenderer>() {
        let mut ids_with_distances = ids
            .iter()
            .map(|id| {
                let world_pos =
                    scene.transform_matrix(*id).unwrap() * Vec4::new(0.0, 0.0, 0.0, 1.0);
                let diff = Vec3::from_vec4(camera_world_pos - world_pos);
                (*id, diff.len_square())
            })
            .collect::<Vec<_>>();

        // closer one comes first
        ids_with_distances.sort_unstable_by(|(_, a), (_, b)| f32::partial_cmp(a, b).unwrap());

        for (id, _) in ids_with_distances {
            let object = scene.find_object_by_id(id).unwrap();

            for renderer in object.find_components_by_type::<StaticMeshRenderer>() {
                if let Some(command) =
                    build_render_command_static_mesh_renderer(ctx.gfx_ctx(), renderer)
                {
                    commands.push(command);
                }
            }
        }
    }

    let mut render_pass = frame.begin_render_pass(
        match camera.clear_mode {
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

    for command in &commands {
        command.render(&mut render_pass);
    }
}

fn render_pass_stage_ui(
    ctx: &Context,
    camera_id: ObjectId,
    surface_texture_view: &TextureView,
    frame: &mut Frame,
    scene: &mut SceneProxy,
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
