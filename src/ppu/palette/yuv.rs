use crate::ppu::palette::rgb::Rgb;

#[derive(Clone, Copy)]
pub struct Yuv {
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

impl Yuv {
    // See https://www.nesdev.org/wiki/NTSC_video#Emulating_in_C++_code
    pub fn to_rgb(self) -> Rgb {
        let red   = clamp(255.0 * (self.y + 1.139883 * self.v));
        let green = clamp(255.0 * (self.y - 0.394642 * self.u - 0.580622 * self.v));
        let blue  = clamp(255.0 * (self.y + 2.032062 * self.u));

        Rgb::new(red, green, blue)
    }
}

fn clamp(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}