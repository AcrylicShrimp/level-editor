mod bone_bind_transform_set;
mod bone_hierarchy;

use crate::{
    context::Context,
    gfx::elements::{PmxModel, PmxModelAnimation},
    scene::{Component, Object, ObjectId, SceneProxy, Transform},
};
use lvl_math::Mat4;
use std::{any::Any, cell::RefMut};

#[derive(Debug)]
pub struct PmxModelAnimator {
    animation: Option<PmxModelAnimation>,
    start_time: Option<f32>,
    is_playing: bool,
    pub loop_enabled: bool,
    // TODO: because MMD is not following ordinal object hierarchy system, we have to manage bones manually by using bone names.
}

impl PmxModelAnimator {
    pub fn new(loop_enabled: bool) -> Self {
        Self {
            animation: None,
            start_time: None,
            is_playing: false,
            loop_enabled,
        }
    }

    pub fn animation(&self) -> Option<&PmxModelAnimation> {
        self.animation.as_ref()
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn elapsed_time(&self, ctx: &Context) -> f32 {
        if !self.is_playing {
            return 0f32;
        }

        match (&self.animation, self.start_time) {
            (Some(animation), Some(start_time)) => {
                let current_time = ctx.time().time().as_secs_f32();
                let elapsed_time = current_time - start_time;

                if animation.total_time() < elapsed_time {
                    0f32
                } else {
                    elapsed_time
                }
            }
            _ => 0f32,
        }
    }

    pub fn set_animation(&mut self, animation: PmxModelAnimation) {
        self.animation = Some(animation);
    }

    pub fn take_animation(&mut self) -> Option<PmxModelAnimation> {
        self.animation.take()
    }

    pub fn play(&mut self, ctx: &Context) {
        if self.animation.is_none() {
            return;
        }

        self.start_time = Some(ctx.time().time().as_secs_f32());
    }

    pub(crate) fn update(&mut self, pmx_model: &mut PmxModel, ctx: &Context) {
        if !self.is_playing {
            return;
        }

        let (animation, mut start_time) = match (&self.animation, self.start_time) {
            (Some(animation), Some(start_time)) => (animation, start_time),
            _ => return,
        };

        let current_time = ctx.time().time().as_secs_f32();
        let elapsed_time = current_time - start_time;

        if animation.total_time() < elapsed_time {
            if self.loop_enabled {
                start_time = current_time;
            } else {
                self.is_playing = false;
                return;
            }
        }

        let bone_key_frame = animation.get_current_bone_key_frame(elapsed_time);
        let morph_key_frame = animation.get_current_morph_key_frame(elapsed_time);

        match (bone_key_frame.current, bone_key_frame.next) {
            (None, None) => {}
            (Some(current), None) | (None, Some(current)) => for element in &current.elements {},
            (Some(current), Some(next)) => {}
        }

        match (morph_key_frame.current, morph_key_frame.next) {
            (None, None) => {}
            (Some(current), None) | (None, Some(current)) => {}
            (Some(current), Some(next)) => {}
        }
    }
}

fn find_bone<'a>(
    root_object_id: ObjectId,
    bone_name: string_interner::DefaultSymbol,
    scene: &'a mut SceneProxy,
) -> Option<(ObjectId, Transform)> {
    let iter = match scene.object_and_children(root_object_id) {
        Some(iter) => iter,
        None => return None,
    };

    for bone_id in iter {
        if scene.name_interned(*bone_id) == bone_name {
            let bone = scene.find_object_by_id(*bone_id).unwrap();
            return Some((*bone_id, bone.transform()));
        }
    }

    None
}

fn bezier_interpolation(x1: f32, x2: f32, y1: f32, y2: f32, t: f32) -> f32 {
    const ITERATIONS: i32 = 15;
    const EPSILON: f32 = 1e-5;

    let mut c = 0.5;
    let mut t = c;
    let mut s = 1.0 - t;

    let mut sst3 = 0f32;
    let mut stt3 = 0f32;
    let mut ttt = 0f32;

    for _ in 0..ITERATIONS {
        sst3 = 3.0 * s * s * t;
        stt3 = 3.0 * s * t * t;
        ttt = t * t * t;

        let ft = sst3 * x1 + stt3 * x2 + ttt - t;

        if ft.abs() < EPSILON {
            break;
        }

        c *= 0.5;

        t += if ft < 0.0 { c } else { -c };
        s = 1.0 - t;
    }

    sst3 * y1 + stt3 * y2 + ttt
}

impl Component for PmxModelAnimator {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
