use crate::{
    context::{driver::Driver, Context},
    scene::Scene,
};
use winit::window::Window;

pub fn late_update(
    window: &Window,
    ctx: &Context,
    scene: &mut Scene,
    driver: &mut Option<Box<dyn Driver>>,
) {
    if let Some(driver) = driver {
        driver.as_mut().on_before_late_update(&ctx, window, scene);
    }

    scene.trigger_late_update();

    if let Some(driver) = driver {
        driver.on_after_late_update(&ctx, window, scene);
    }
}
