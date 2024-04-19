use crate::context::{driver::Driver, Context};
use winit::window::Window;

pub fn late_update(window: &Window, ctx: &Context, driver: &mut Option<Box<dyn Driver>>) {
    if let Some(driver) = driver {
        driver.as_mut().on_before_late_update(&ctx, window);
    }

    // TODO: perform actual late update here

    if let Some(driver) = driver {
        driver.on_after_late_update(&ctx, window);
    }
}
