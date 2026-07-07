use egui::{Context, Ui};

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::PixelBuffer;

pub struct PatternSourceRenderer;

impl PatternSourceRenderer {
    pub fn new() -> Self { PatternSourceRenderer }
}

impl WindowRenderer for PatternSourceRenderer {
    fn name(&self) -> String {
        "Pattern Source".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _ui: &mut Ui, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixel_buffer: &mut PixelBuffer) {
        let Some(nes) = &world.nes else {
            return;
        };

        nes.ppu().pattern_source_debug_buffer().copy_to_rgba_buffer(pixel_buffer.frame_mut());
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}