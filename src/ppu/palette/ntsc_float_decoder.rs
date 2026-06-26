use crate::master_clock::MasterClock;
use crate::ppu::palette::composite_decoder::CompositeDecoder;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::register::ppu_registers::Emphasis;

const WAVELENGTH: u64 = 12;// Terminated voltage levels

// Reference implementation: https://www.nesdev.org/wiki/NTSC_video#Emulating_in_C++_code
pub struct NtscFloatDecoder {
    phase: u8, // 0-11
}

impl NtscFloatDecoder {
    pub fn new() -> Self {
        NtscFloatDecoder {
            phase: 0,
        }
    }
}

impl CompositeDecoder for NtscFloatDecoder {
    fn start_scanline(&mut self, clock: &MasterClock) {
        self.phase = (clock.ppu_clock().total_cycles() % WAVELENGTH) as u8;
    }

    fn decode_to_rgb(&self, _color: Color, _emphasis: Emphasis) -> Rgb {
        Rgb::BLACK
    }
}