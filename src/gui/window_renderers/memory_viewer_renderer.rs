use egui::{Context, Ui};

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::bus::AddressBusType;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::ppu::render::frame::Frame;

pub struct MemoryViewerRenderer;

impl MemoryViewerRenderer {
    const WIDTH: usize = 700;
    const HEIGHT: usize = 400;
}

impl WindowRenderer for MemoryViewerRenderer {
    fn name(&self) -> String {
        "Memory Viewer".to_string()
    }

    fn ui(&mut self, _ctx: &Context, ui: &mut Ui, world: &mut World, _: &mut Frame) -> FlowControl {
        let Some(nes) = &world.nes else {
            return FlowControl::CONTINUE;
        };

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(16)
                    .spacing([0.0, 0.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for mem_index in 0..=u16::MAX {
                            let value = nes.bus().cpu_peek(nes.mapper(), AddressBusType::Cpu, CpuAddress::new(mem_index));
                            let _ = ui.button(format!("{value:02X}")).on_hover_text(format!("0x{mem_index:04X}"));
                            if mem_index % 0x10 == 0x0F {
                                ui.end_row();
                            }
                        }
                    });
            })
        });

        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, _frame: &mut Frame) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}