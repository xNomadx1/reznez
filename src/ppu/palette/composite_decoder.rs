use crate::ppu::palette::color::Color;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::register::ppu_registers::Mask;

pub trait CompositeDecoder {
    fn decode_to_rgb(&self, color: Color, mask: Mask) -> Rgb;

    fn decode_to_rgbt(&self, color_t: ColorT, mask: Mask) -> Rgbt {
        match color_t {
            ColorT::Transparent => Rgbt::Transparent,
            ColorT::Opaque(color) => Rgbt::Opaque(self.decode_to_rgb(color, mask)),
        }
    }
}