use crate::master_clock::MasterClock;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::composite_decoder::CompositeDecoder;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pixel_index::PixelIndex;
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::Frame;

pub struct SystemPaletteDecoder {
    system_palette: SystemPalette,
}

impl SystemPaletteDecoder {
    pub fn new(system_palette: SystemPalette) -> Self {
        Self { system_palette }
    }

    pub fn system_palette(&self) -> &SystemPalette {
        &self.system_palette
    }
}

impl CompositeDecoder for SystemPaletteDecoder {
    fn set_color(&mut self, frame: &mut Frame, clock: &MasterClock, color: Color, emphasis: Emphasis) {
        let index = PixelIndex::try_from_clock(clock.ppu_clock()).unwrap();
        let rgb = self.system_palette.lookup_rgb(color, emphasis);
        frame.set_pixel(index.column.to_usize(), index.row.to_usize(), rgb);
    }
}