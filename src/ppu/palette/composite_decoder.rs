use crate::master_clock::MasterClock;
use crate::ppu::palette::color::Color;
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::Frame;

pub trait CompositeDecoder {
    fn set_color(&mut self, frame: &mut Frame, clock: &MasterClock, color: Color, emphasis: Emphasis);
}