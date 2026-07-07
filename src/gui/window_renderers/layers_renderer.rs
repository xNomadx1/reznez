use egui::{Context, Ui};

use crate::bus::Bus;
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::{DebugBuffer, FrameBuffer, PixelBuffer};

const TOP_MENU_BAR_HEIGHT: usize = 24;

pub struct LayersRenderer {
    frame: FrameBuffer<ColorT>,
    buffer: DebugBuffer<{ LayersRenderer::WIDTH }, { LayersRenderer::HEIGHT }>,
}

impl LayersRenderer {
    const WIDTH: usize = 517;
    const HEIGHT: usize = 485 + TOP_MENU_BAR_HEIGHT;

    pub fn new() -> LayersRenderer {
        LayersRenderer {
            frame: FrameBuffer::default(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl WindowRenderer for LayersRenderer {
    fn name(&self) -> String {
        "Layers".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _ui: &mut Ui, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixel_buffer: &mut PixelBuffer) {
        let Some(nes) = &world.nes else {
            return;
        };

        self.buffer.place_frame(0, TOP_MENU_BAR_HEIGHT, nes.frame());
        self.buffer.place_frame(
            261,
            TOP_MENU_BAR_HEIGHT,
            // FIXME: This no longer places the background. Add a separate Frame to Ppu.
            &nes.frame(),
        );

        let bus = nes.bus();

        self.frame.clear();
        bus.oam.only_front_sprites().render(bus, &mut self.frame);
        self.buffer.place_frame_buffer_with(0, 245 + TOP_MENU_BAR_HEIGHT, &self.frame, |color_t| color_t_to_rgb(&bus, color_t));

        self.frame.clear();
        bus.oam.only_back_sprites().render(bus, &mut self.frame);
        self.buffer.place_frame_buffer_with(261, 245 + TOP_MENU_BAR_HEIGHT, &self.frame, |color_t| color_t_to_rgb(&bus, color_t));

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