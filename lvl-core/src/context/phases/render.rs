mod collects;
mod render_command;
mod render_pmx_model_renderer;

use self::{
    collects::collect_components,
    render_pmx_model_renderer::build_render_command_pmx_model_renderer,
};
use super::common::get_all_cameras;
use crate::{
    context::{driver::Driver, Context},
    gfx::{ClearMode, Frame, InstanceDataProvider, RenderPassTarget},
    scene::{
        components::{Camera, CameraClearMode, Light, PmxModelRenderer},
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
            let screen_size = ctx.screen_size();

            let camera = proxy
                .find_object_by_id(camera_id)
                .unwrap()
                .find_component_by_type::<Camera>()
                .unwrap();
            let camera_transform_matrix = proxy.transform_matrix(camera_id).unwrap();
            let camera_world_pos = camera_transform_matrix.split_translation();
            let camera_projection_matrix = camera.projection_mode.to_mat4(
                screen_size.width as f32 / screen_size.height as f32,
                &camera_transform_matrix.inversed(),
            );

            ctx.gfx_ctx()
                .uniform_bind_group_provider
                .update_camera_matrix(
                    &camera_projection_matrix,
                    camera_world_pos,
                    camera_transform_matrix,
                    &ctx.gfx_ctx().queue,
                );

            render_pass_stage_opaque(ctx, camera_id, &surface_texture_view, &mut frame, proxy);
            // render_pass_stage_ui(ctx, camera_id, &surface_texture_view, &mut frame, proxy);
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

    if let Some(ids) = scene.find_object_ids_by_component_type::<PmxModelRenderer>() {
        let mut renderers_and_distances = Vec::with_capacity(ids.len());

        for id in ids {
            let object = scene.find_object_by_id(*id).unwrap();
            let world_pos = scene.transform_matrix(*id).unwrap() * Vec4::new(0.0, 0.0, 0.0, 1.0);
            let diff = Vec3::from_vec4(camera_world_pos - world_pos);
            let distance = diff.len_square();
            let renderers = object.find_components_by_type::<PmxModelRenderer>();

            for renderer in renderers {
                renderers_and_distances.push((distance, *id, renderer));
            }
        }

        // closer one comes first
        renderers_and_distances
            .sort_unstable_by(|(a, _, _), (b, _, _)| f32::partial_cmp(a, b).unwrap());

        for (_, id, renderer) in renderers_and_distances {
            let transform_matrix = scene.transform_matrix(id).unwrap();
            commands.extend(build_render_command_pmx_model_renderer(
                transform_matrix,
                renderer,
                &InstanceDataProvider,
                ctx.gfx_ctx(),
            ));
        }
    }

    let global_texture_set = ctx.gfx_ctx().global_texture_set.borrow();
    let depth_texture_view = &global_texture_set.depth_stencil.texture_view;

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
        Some(RenderPassTarget {
            view: depth_texture_view,
            writable: true,
        }),
    );

    let bind_group = ctx.gfx_ctx().uniform_bind_group_provider.bind_group();

    for command in &commands {
        command.render(&mut render_pass, bind_group);
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

fn _test_render(
    window: &Window,
    ctx: &Context,
    scene: &mut Scene,
    driver: &mut Option<Box<dyn Driver>>,
) {
    let bind_group = ctx.gfx_ctx().uniform_bind_group_provider.bind_group();

    let surface_texture = ctx.gfx_ctx().obtain_surface_view().unwrap();
    let surface_texture_view = surface_texture.texture.create_view(&Default::default());

    let global_texture_set = ctx.gfx_ctx().global_texture_set.borrow();
    let depth_texture_view = &global_texture_set.depth_stencil.texture_view;

    let mut frame = ctx.gfx_ctx().begin_frame();

    scene.with_proxy(|scene| {
        let cameras = collect_components::<Camera>(scene);
        let lights = collect_components::<Light>(scene);
        let mut pmx_model_renderers = collect_components::<PmxModelRenderer>(scene);

        for camera in &cameras {
            let camera_world_position = camera.world_position();

            pmx_model_renderers.sort_unstable_by(|a, b| {
                let a = (a.world_position() - camera_world_position).len_square();
                let b = (b.world_position() - camera_world_position).len_square();
                f32::partial_cmp(&a, &b).unwrap()
            });

            let mut render_pass = frame.begin_render_pass(
                match camera.component.clear_mode {
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
                Some(RenderPassTarget {
                    view: depth_texture_view,
                    writable: true,
                }),
            );

            for renderer in &pmx_model_renderers {
                let pipelines = renderer.component.construct_render_pipelines(
                    InstanceDataProvider.instance_data_size(),
                    InstanceDataProvider.instance_data_attributes(),
                    ctx.gfx_ctx(),
                );
            }

            for light in &lights {}
        }
    });
}
