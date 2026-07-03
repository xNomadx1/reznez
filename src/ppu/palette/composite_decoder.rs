use crate::master_clock::MasterClock;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::Frame;

pub trait CompositeDecoder {
    fn set_color(&mut self, frame: &mut Frame, clock: &MasterClock, color: Color, emphasis: Emphasis);

    fn decode_to_rgb(&self, color: Color, emphasis: Emphasis) -> Rgb;

    fn decode_to_rgbt(&self, color_t: ColorT, emphasis: Emphasis) -> Rgbt {
        match color_t {
            ColorT::Transparent => Rgbt::Transparent,
            ColorT::Opaque(color) => Rgbt::Opaque(self.decode_to_rgb(color, emphasis)),
        }
    }

    fn finalize_scanline(&self, frame: &mut Frame, clock: &MasterClock);
}