use egui::{Context, Ui};
use winit::dpi::Position;

use crate::gui::world::World;
use crate::ppu::render::frame::Frame;

pub trait WindowRenderer {
    fn name(&self) -> String;
    fn ui(&mut self, ctx: &Context, ui: &mut Ui, world: &mut World) -> FlowControl;
    fn render(&mut self, _world: &mut World, _frame: &mut Frame) {
        // Most debug windows don't need to render any pixel graphics.
    }
    fn toggle_pause(&mut self) {}
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub type WindowArgs = (Box<dyn WindowRenderer>, Position, u64);

pub struct FlowControl {
    pub window_args: Option<WindowArgs>,
    pub should_close_window: bool,
}

impl FlowControl {
    pub const CONTINUE: Self = Self { window_args: None, should_close_window: false };

    pub fn spawn_window(window_args: WindowArgs) -> Self {
        Self {
            window_args: Some(window_args),
            should_close_window: false,
        }
    }
}
