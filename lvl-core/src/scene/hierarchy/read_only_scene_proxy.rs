use super::SceneProxy;
use std::ops::Deref;

pub struct ReadOnlySceneProxy<'scene, 'window> {
    scene_proxy: SceneProxy<'scene, 'window>,
}

impl<'scene, 'window> ReadOnlySceneProxy<'scene, 'window> {
    pub(crate) fn new(scene_proxy: SceneProxy<'scene, 'window>) -> Self {
        Self { scene_proxy }
    }

    pub fn scene(&self) -> &SceneProxy {
        &self.scene_proxy
    }
}

impl<'scene, 'window> Deref for ReadOnlySceneProxy<'scene, 'window> {
    type Target = SceneProxy<'scene, 'window>;

    fn deref(&self) -> &Self::Target {
        &self.scene_proxy
    }
}
