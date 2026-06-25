use crate::ppu::palette::composite_decoder::CompositeDecoder;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::register::ppu_registers::Mask;

pub struct NtscFloatDecoder {}

impl NtscFloatDecoder {
    pub fn new() -> Self {
        NtscFloatDecoder {}
    }
}

impl CompositeDecoder for NtscFloatDecoder {
    fn decode_to_rgb(&self, _color: Color, _mask: Mask) -> Rgb {
        Rgb::BLACK
    }
}