use egui::{Context, Ui};

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;

pub struct DisplaySettingsRenderer;

impl DisplaySettingsRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new() -> Self {
        Self
    }
}

impl WindowRenderer for DisplaySettingsRenderer {
    fn name(&self) -> String {
        "Display Settings".to_string()
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut Ui, world: &mut World) -> FlowControl {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(nes) = &mut world.nes {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(&mut nes.bus_mut().composite_decoders.show_overscan, "Show overscan");
                        ui.checkbox(&mut nes.bus_mut().composite_decoders.use_ntsc_float_decoder, "Use NTSC filter");
                        ui.end_row();
                    });
            } else {
                ui.label("Load a ROM to change display settings.");

            }
        });

        FlowControl::CONTINUE
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}
