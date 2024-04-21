use crate::{
    context::{driver::Driver, Context},
    scene::Scene,
};
use winit::window::Window;

pub fn update(
    window: &Window,
    ctx: &Context,
    scene: &mut Scene,
    driver: &mut Option<Box<dyn Driver>>,
) {
    if let Some(driver) = driver {
        driver.on_before_update(&ctx, window, scene);
    }

    scene.trigger_update();

    if let Some(driver) = driver {
        driver.on_after_update(&ctx, window, scene);
    }
}
