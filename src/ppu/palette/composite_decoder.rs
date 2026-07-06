use crate::master_clock::MasterClock;
use crate::ppu::palette::color::Color;
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::Frame;

pub trait CompositeDecoder {
    fn frame(&self) -> &Frame;
    fn set_color(&mut self, clock: &MasterClock, color: Color, emphasis: Emphasis);
}