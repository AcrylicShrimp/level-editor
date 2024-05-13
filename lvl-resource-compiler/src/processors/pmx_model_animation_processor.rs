use super::Processor;
use anyhow::Error as AnyError;
use lvl_math::{Quat, Vec3};
use lvl_resource::{
    PmxModelAnimationBoneBezier, PmxModelAnimationBoneKeyFrame,
    PmxModelAnimationBoneKeyFrameElement, PmxModelAnimationMorphKeyFrame,
    PmxModelAnimationMorphKeyFrameElement, PmxModelAnimationSource, Resource, ResourceKind,
};
use lvl_vmd::Vmd;
use std::{collections::HashMap, path::Path};

pub struct PmxModelAnimationProcessor;

impl Processor for PmxModelAnimationProcessor {
    type Metadata = ();

    fn extension() -> &'static [&'static str] {
        &["vmd"]
    }

    fn process(file: &Path, _metadata: Option<&Self::Metadata>) -> Result<Vec<Resource>, AnyError> {
        let vmd = {
            let content = std::fs::read(file)?;
            Vmd::parse(&content)?
        };

        let mut bone_key_frames = HashMap::<u32, Vec<_>>::new();

        for key_frame in &vmd.bone_key_frames {
            bone_key_frames
                .entry(key_frame.frame_index)
                .or_default()
                .push(PmxModelAnimationBoneKeyFrameElement {
                    bone_name: key_frame.bone_name.clone(),
                    translation: Vec3::new(
                        key_frame.translation.x,
                        key_frame.translation.y,
                        key_frame.translation.z,
                    ),
                    rotation: Quat::new(
                        key_frame.rotation.x,
                        key_frame.rotation.y,
                        key_frame.rotation.z,
                        key_frame.rotation.w,
                    ),
                    bezier: PmxModelAnimationBoneBezier {
                        x_axis: [
                            key_frame.bezier.data[0],
                            key_frame.bezier.data[8],
                            key_frame.bezier.data[4],
                            key_frame.bezier.data[12],
                        ],
                        y_axis: [
                            key_frame.bezier.data[16],
                            key_frame.bezier.data[24],
                            key_frame.bezier.data[20],
                            key_frame.bezier.data[28],
                        ],
                        z_axis: [
                            key_frame.bezier.data[32],
                            key_frame.bezier.data[40],
                            key_frame.bezier.data[36],
                            key_frame.bezier.data[44],
                        ],
                        rotation: [
                            key_frame.bezier.data[48],
                            key_frame.bezier.data[56],
                            key_frame.bezier.data[52],
                            key_frame.bezier.data[60],
                        ],
                    },
                });
        }

        let mut morph_key_frames = HashMap::<u32, Vec<_>>::new();

        for key_frame in &vmd.morph_key_frames {
            morph_key_frames
                .entry(key_frame.frame_index)
                .or_default()
                .push(PmxModelAnimationMorphKeyFrameElement {
                    morph_name: key_frame.morph_name.clone(),
                    weight: key_frame.weight,
                });
        }

        let mut bone_key_frames = bone_key_frames
            .into_iter()
            .map(|(frame_index, elements)| PmxModelAnimationBoneKeyFrame {
                frame_index,
                elements,
            })
            .collect::<Vec<_>>();

        let mut morph_key_frames = morph_key_frames
            .into_iter()
            .map(|(frame_index, elements)| PmxModelAnimationMorphKeyFrame {
                frame_index,
                elements,
            })
            .collect::<Vec<_>>();

        bone_key_frames.sort_unstable_by_key(|kf| kf.frame_index);
        morph_key_frames.sort_unstable_by_key(|kf| kf.frame_index);

        Ok(vec![Resource {
            name: file.file_stem().unwrap().to_string_lossy().to_string(),
            kind: ResourceKind::PmxModelAnimation(PmxModelAnimationSource::new(
                bone_key_frames,
                morph_key_frames,
            )),
        }])
    }
}
