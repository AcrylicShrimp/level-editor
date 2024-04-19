use super::Context;
use winit::window::Window;

pub trait Driver
where
    Self: 'static,
{
    fn on_init(&mut self, _context: &Context, _window: &Window) {}
    fn on_finish(&mut self, _context: &Context, _window: &Window) {}
    fn on_before_update(&mut self, _context: &Context, _window: &Window) {}
    fn on_after_update(&mut self, _context: &Context, _window: &Window) {}
    fn on_before_late_update(&mut self, _context: &Context, _window: &Window) {}
    fn on_after_late_update(&mut self, _context: &Context, _window: &Window) {}
    fn on_before_render(&mut self, _context: &Context, _window: &Window) {}
    fn on_after_render(&mut self, _context: &Context, _window: &Window) {}
}
