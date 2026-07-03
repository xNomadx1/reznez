use std::f32::consts::PI;

use crate::master_clock::MasterClock;
use crate::ppu::palette::composite_decoder::CompositeDecoder;
use crate::ppu::palette::color::{Brightness, Color, Hue};
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::yuv::Yuv;
use crate::ppu::pixel_index::{PixelColumn, PixelIndex, PixelRow};
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::frame::Frame;

const WAVELENGTH: u64 = 12;// Terminated voltage levels

const LEVELS: [f32; 16] = [
    0.228, 0.312, 0.552, 0.880, // Signal low
    0.616, 0.840, 1.100, 1.100, // Signal high
    0.192, 0.256, 0.448, 0.712, // Signal low, attenuated
    0.500, 0.676, 0.896, 0.896  // Signal high, attenuated
];

// Reference implementation: https://www.nesdev.org/wiki/NTSC_video#Emulating_in_C++_code
#[allow(dead_code)]
pub struct NtscFloatDecoder {
    scanline_start_phase: usize, // 0-11
    signal_levels: [f32; 8 * 256], // All signal levels for a single scanline, 8 per pixel
}

impl NtscFloatDecoder {
    pub fn new() -> Self {
        Self {
            scanline_start_phase: 0,
            signal_levels: [0.0; 8 * 256],
        }
    }
}

impl CompositeDecoder for NtscFloatDecoder {
    fn set_color(&mut self, frame: &mut Frame, clock: &MasterClock, color: Color, emphasis: Emphasis) {
        let ppuclock = clock.ppu_clock();
        if ppuclock.cycle() == 1 {
            self.scanline_start_phase = (ppuclock.total_cycles() % WAVELENGTH) as usize;
        }

        self.set_pixel_signal_levels(clock, color, emphasis);

        if ppuclock.cycle() == 256 {
            self.finalize_scanline(frame, clock);
        }
    }

    fn decode_to_rgb(&self, _color: Color, _emphasis: Emphasis) -> Rgb {
        //self.set_pixel_signal_levels(color, emphasis);
        Rgb::BLACK
    }

    fn finalize_scanline(&self, frame: &mut Frame, clock: &MasterClock) {
        const WIDTH: usize = 256;
        const SAMPLE_COUNT: usize = 8 * 256;

        for x in 0..WIDTH {
            let center = x * SAMPLE_COUNT / WIDTH;
            let begin = center.saturating_sub(6);
            let end = std::cmp::max(center + 6, SAMPLE_COUNT);

            let mut y = 0.0;
            let mut u = 0.0;
            let mut v = 0.0;
            for p in begin..end {
                let level = self.signal_levels[p] / 12.0;
                y += level;

                // Magic constants explanation:
                // * 2.0: Saturation correction for integral of sin(2*PI*x)^2
                // + 3.0: Carrier reference phase is off by 90 degrees
                // - 0.5: Carrier phase is additionally off by 15 degrees
                let angle = PI * ((self.scanline_start_phase + p) as f32 + 3.0 - 0.5) / 6.0;
                u  += level * angle.sin() * 2.0;
                v  += level * angle.cos() * 2.0;
            }

            let pixel_index = PixelIndex {
                column: PixelColumn::new(x as u8),
                row: PixelRow::from_scanline(clock.ppu_clock().scanline()).unwrap(),
            };
            let yuv = Yuv { y, u, v };
            frame.set_pixel(pixel_index, yuv.to_rgb());
        }
    }
}

#[allow(dead_code)]
impl NtscFloatDecoder {
    fn set_pixel_signal_levels(&mut self, clock: &MasterClock, color: Color, emphasis: Emphasis) {
        let column = PixelIndex::try_from_clock(clock.ppu_clock()).unwrap().column;
        let phase = 8 * clock.ppu_clock().total_cycles();
        for p in 0..8 {
            let mut signal = self.signal(phase + p, color, emphasis);
            // TODO: Add low pass filtering to emulate differential phase distortion.

            // Normalize the signal to be between 0 and 1
            const BLACK: f32 = 0.312;
            const WHITE: f32 = 1.100;
            signal = (signal - BLACK) / (WHITE - BLACK);
            self.signal_levels[8 * column.to_usize() + p as usize] = signal;
        }
    }

    fn signal(&self, phase: u64, color: Color, emphasis: Emphasis) -> f32 {
        let mut brightness = color.brightness();
        let hue = color.hue();
        if matches!(hue, Hue::DarkGray | Hue::Black | Hue::ExtraBlack) {
            brightness = Brightness::Low;
        }

        let in_phase = |hue: Hue| -> bool {
            ((hue as u64) + phase) % 12 < 6
        };

        let mut is_attenuated =
               (emphasis.red()   && in_phase(Hue::Olive))
            || (emphasis.green() && in_phase(Hue::Magenta))
            || (emphasis.blue()  && in_phase(Hue::Cyan));
        if matches!(hue, Hue::Black | Hue::ExtraBlack) {
            is_attenuated = false;
        }

        let attenuation = if is_attenuated { 8 } else { 0 };

        let brightness = brightness as usize;
        let mut low  = LEVELS[0 + brightness + attenuation];
        let mut high = LEVELS[4 + brightness + attenuation];
        // Grays and Blacks have signal levels that don't depend on phase.
        if hue == Hue::Gray { low = high; }
        if matches!(hue, Hue::DarkGray | Hue::Black | Hue::ExtraBlack) { high = low; }

        if in_phase(hue) { high } else { low }
    }
}