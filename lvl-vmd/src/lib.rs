mod cursor;
mod parse;
mod primitives;
mod vmd_bone_key_frame;
mod vmd_camera_key_frame;
mod vmd_header;
mod vmd_light_key_frame;
mod vmd_morph_key_frame;
mod vmd_primitives;

use cursor::Cursor;
use parse::Parse;
use std::fmt::Display;
use thiserror::Error;
pub use vmd_bone_key_frame::*;
pub use vmd_camera_key_frame::*;
pub use vmd_header::*;
pub use vmd_light_key_frame::*;
pub use vmd_morph_key_frame::*;

#[derive(Error, Debug)]
pub enum VmdParseError {
    #[error("failed to parse VMD header: {0}")]
    VmdHeaderParseError(#[from] VmdHeaderParseError),
    #[error("failed to parse VMD bone key frame: {0}")]
    VmdBoneKeyFrameParseError(#[from] VmdBoneKeyFrameParseError),
    #[error("failed to parse VMD morph key frame: {0}")]
    VmdMorphKeyFrameParseError(#[from] VmdMorphKeyFrameParseError),
    #[error("failed to parse VMD camera key frame: {0}")]
    VmdCameraKeyFrameParseError(#[from] VmdCameraKeyFrameParseError),
    #[error("failed to parse VMD light key frame: {0}")]
    VmdLightKeyFrameParseError(#[from] VmdLightKeyFrameParseError),
}

#[derive(Debug, Clone)]
pub struct Vmd {
    pub header: VmdHeader,
    pub bone_key_frames: Vec<VmdBoneKeyFrame>,
    pub morph_key_frames: Vec<VmdMorphKeyFrame>,
    pub camera_key_frames: Vec<VmdCameraKeyFrame>,
    pub light_key_frames: Vec<VmdLightKeyFrame>,
}

impl Vmd {
    pub fn parse(buf: impl AsRef<[u8]>) -> Result<Self, VmdParseError> {
        let mut cursor = Cursor::new(buf.as_ref());

        let header = VmdHeader::parse(&mut cursor)?;
        let bone_key_frames = Vec::parse(&mut cursor)?;
        let morph_key_frames = Vec::parse(&mut cursor)?;
        let camera_key_frames = Vec::parse(&mut cursor)?;
        let light_key_frames = Vec::parse(&mut cursor)?;

        Ok(Self {
            header,
            bone_key_frames,
            morph_key_frames,
            camera_key_frames,
            light_key_frames,
        })
    }
}

impl Display for Vmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "VMD {}", self.header.version)?;
        writeln!(f, "  model name: {}", self.header.model_name)?;
        writeln!(f, "  bone key frames: {}", self.bone_key_frames.len())?;
        writeln!(f, "  morph key frames: {}", self.morph_key_frames.len())?;
        writeln!(f, "  camera key frames: {}", self.camera_key_frames.len())?;
        writeln!(f, "  light key frames: {}", self.light_key_frames.len())?;
        Ok(())
    }
}
