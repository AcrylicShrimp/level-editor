use lvl_resource::{
    PmxModelAnimationBoneKeyFrame, PmxModelAnimationMorphKeyFrame, PmxModelAnimationSource,
};

#[derive(Debug)]
pub struct PmxModelAnimation {
    bone_key_frames: Vec<PmxModelAnimationBoneKeyFrame>,
    morph_key_frames: Vec<PmxModelAnimationMorphKeyFrame>,
    total_time: f32,
    fps: f32,
}

impl PmxModelAnimation {
    pub fn load_from_source(source: &PmxModelAnimationSource, fps: f32) -> Self {
        let max_bone_key_frame = source
            .bone_key_frames()
            .last()
            .map_or(0, |kf| kf.frame_index);
        let max_morph_key_frame = source
            .morph_key_frames()
            .last()
            .map_or(0, |kf| kf.frame_index);
        let total_time = (max_bone_key_frame + max_morph_key_frame) as f32 / fps;

        Self {
            fps,
            total_time,
            bone_key_frames: source.bone_key_frames().to_vec(),
            morph_key_frames: source.morph_key_frames().to_vec(),
        }
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn get_current_bone_key_frame(&self, play_time: f32) -> CurrentBoneKeyFrame {
        let frame_index = (play_time * self.fps) as u32;

        match self
            .bone_key_frames
            .binary_search_by_key(&frame_index, |kf| kf.frame_index)
        {
            Ok(index) => CurrentBoneKeyFrame {
                weight: 0f32,
                current: self.bone_key_frames.get(index),
                next: self.bone_key_frames.get(index + 1),
            },
            Err(index) => match index {
                0 => CurrentBoneKeyFrame {
                    weight: 0f32,
                    current: self.bone_key_frames.get(index),
                    next: self.bone_key_frames.get(index + 1),
                },
                index if index == self.bone_key_frames.len() => CurrentBoneKeyFrame {
                    weight: 0f32,
                    current: self.bone_key_frames.last(),
                    next: None,
                },
                index => {
                    let current = &self.bone_key_frames[index - 1];
                    let next = &self.bone_key_frames[index];

                    CurrentBoneKeyFrame {
                        weight: (frame_index - current.frame_index) as f32
                            / (next.frame_index - current.frame_index) as f32,
                        current: Some(current),
                        next: Some(next),
                    }
                }
            },
        }
    }

    pub fn get_current_morph_key_frame(&self, play_time: f32) -> CurrentMorphKeyFrame {
        let frame_index = (play_time * self.fps) as u32;

        match self
            .morph_key_frames
            .binary_search_by_key(&frame_index, |kf| kf.frame_index)
        {
            Ok(index) => CurrentMorphKeyFrame {
                weight: 0f32,
                current: self.morph_key_frames.get(index),
                next: self.morph_key_frames.get(index + 1),
            },
            Err(index) => match index {
                0 => CurrentMorphKeyFrame {
                    weight: 0f32,
                    current: self.morph_key_frames.get(index),
                    next: self.morph_key_frames.get(index + 1),
                },
                index if index == self.morph_key_frames.len() => CurrentMorphKeyFrame {
                    weight: 0f32,
                    current: self.morph_key_frames.last(),
                    next: None,
                },
                index => {
                    let current = &self.morph_key_frames[index - 1];
                    let next = &self.morph_key_frames[index];

                    CurrentMorphKeyFrame {
                        weight: (frame_index - current.frame_index) as f32
                            / (next.frame_index - current.frame_index) as f32,
                        current: Some(current),
                        next: Some(next),
                    }
                }
            },
        }
    }
}

pub struct CurrentBoneKeyFrame<'a> {
    /// Represents how much of the next frame is shown.
    ///
    /// - `0` if the given frame is at exactly the current frame.
    /// - `1` if the given frame is at exactly the next frame.
    ///
    /// It is intended to be used to interpolate between the current and next frame.
    pub weight: f32,
    pub current: Option<&'a PmxModelAnimationBoneKeyFrame>,
    pub next: Option<&'a PmxModelAnimationBoneKeyFrame>,
}

pub struct CurrentMorphKeyFrame<'a> {
    /// Represents how much of the next frame is shown.
    ///
    /// - `0` if the given frame is at exactly the current frame.
    /// - `1` if the given frame is at exactly the next frame.
    ///
    /// It is intended to be used to interpolate between the current and next frame.
    pub weight: f32,
    pub current: Option<&'a PmxModelAnimationMorphKeyFrame>,
    pub next: Option<&'a PmxModelAnimationMorphKeyFrame>,
}
