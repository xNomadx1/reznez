use crate::gui::gui::Gui;
use crate::nes::Nes;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::Frame;

pub struct NoGui;

impl Gui for NoGui {
    fn run(&mut self, nes: Option<Nes>) {
        let mut nes = nes.expect("ROM to be specified when nogui mode is specified.");
        let mut frame = Frame::dummy(PixelColumn::COLUMN_COUNT * PixelRow::ROW_COUNT);
        loop {
            nes.step_frame(&mut frame);
        }
    }
}
