use egui::{Context, Ui};

use crate::bus::Bus;
use crate::gui::debug_screens::name_table::NameTable;
use crate::gui::debug_screens::pattern_table::PatternTable;
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::{DebugBuffer, FrameBuffer, PixelBuffer};

pub struct NameTableRenderer {
    frame: FrameBuffer<ColorT>,
    buffer: DebugBuffer<{ NameTableRenderer::WIDTH }, { NameTableRenderer::HEIGHT }>,
}

impl NameTableRenderer {
    const WIDTH: usize = 2 * 256 + 2;
    const HEIGHT: usize = 2 * 240 + 2;

    pub fn new() -> NameTableRenderer {
        NameTableRenderer {
            frame: FrameBuffer::default(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl WindowRenderer for NameTableRenderer {
    fn name(&self) -> String {
        "Name Tables".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _ui: &mut Ui, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    #[rustfmt::skip]
    fn render(&mut self, world: &mut World, pixel_buffer: &mut PixelBuffer) {
        let Some(nes) = &world.nes else {
            return;
        };

        let x = usize::from(nes.bus().ppu_regs.x_scroll().to_u8());
        let y = usize::from(nes.bus().ppu_regs.next_address.y_scroll().to_u8());
        let bus = &mut nes.bus();

        let width = NameTableRenderer::WIDTH;
        let height = NameTableRenderer::HEIGHT;
        // Clear any junk out of the outer border.
        self.buffer.place_wrapping_horizontal_line(0, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_horizontal_line(height, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(0, 0, height, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(width, 0, height, Rgb::new(255, 255, 255));

        let background_table = PatternTable::background_side(bus);

        NameTable::new(bus.raw_name_table(NameTableQuadrant::TopLeft))
            .render(&background_table, bus.palette_ram(), &mut self.frame);
        self.buffer.place_frame_buffer_with(1, 1, &self.frame, |color_t| color_t_to_rgb(&bus, color_t));
        NameTable::new(bus.raw_name_table(NameTableQuadrant::TopRight))
            .render(&background_table, bus.palette_ram(), &mut self.frame);
        self.buffer.place_frame_buffer_with(257, 1, &self.frame, |color_t| color_t_to_rgb(&bus, color_t));
        NameTable::new(bus.raw_name_table(NameTableQuadrant::BottomLeft))
            .render(&background_table, bus.palette_ram(), &mut self.frame);
        self.buffer.place_frame_buffer_with(1, 241, &self.frame, |color_t| color_t_to_rgb(&bus, color_t));
        NameTable::new(bus.raw_name_table(NameTableQuadrant::BottomRight))
            .render(&background_table, bus.palette_ram(), &mut self.frame);
        self.buffer.place_frame_buffer_with(257, 241, &self.frame, |color_t| color_t_to_rgb(&bus, color_t));

        self.buffer.place_wrapping_horizontal_line(y, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_horizontal_line(y + 241, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x, y, y + 241, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x + 257, y, y + 241, Rgb::new(255, 0, 0));

        self.buffer.copy_to_rgba_buffer(pixel_buffer.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

fn color_t_to_rgb(bus: &Bus, color_t: ColorT) -> Rgb {
    bus.composite_decoders.system_palette_decoder.system_palette()
        .lookup_rgbt(color_t, Emphasis::OFF)
        .to_rgb()
        .unwrap_or(Rgb::BLACK)
}