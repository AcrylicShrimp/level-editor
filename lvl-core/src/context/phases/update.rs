use crate::context::{driver::Driver, Context};
use winit::window::Window;

pub fn update(window: &Window, ctx: &Context, driver: &mut Option<Box<dyn Driver>>) {
    if let Some(driver) = driver {
        driver.on_before_update(&ctx, window);
    }

    // TODO: perform actual update here

    if let Some(driver) = driver {
        driver.on_after_update(&ctx, window);
    }
}
