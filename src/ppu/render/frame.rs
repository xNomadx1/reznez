use std::ops::{Index, IndexMut};

use enum_iterator::all;
use pixels::{Pixels, PixelsContext, wgpu};

use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::gui::debug_screens::pattern_table::Tile;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelIndex, PixelRow, RowInTile};
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::ppm::Ppm;

pub enum Frame {
    Dummy {
        buffer: Vec<u8>,
        pixel_width: usize,
        pixel_height: usize,
    },
    WindowBacked(Pixels<'static>),
}

impl Frame {
    pub fn new(pixels: Pixels<'static>) -> Self {
        Self::WindowBacked(pixels)
    }

    pub fn dummy(pixel_width: usize, pixel_height: usize) -> Self {
        Self::Dummy {
            buffer: vec![0; 4 * pixel_width * pixel_height],
            pixel_width,
            pixel_height,
        }
    }

    pub fn exact_sized() -> Self {
        Self::dummy(PixelColumn::COLUMN_COUNT, PixelRow::ROW_COUNT)
    }

    pub fn frame(&self) -> &[u8] {
        match self {
            Self::Dummy { buffer, .. } => &buffer,
            Self::WindowBacked(pixels) => pixels.frame(),
        }
    }

    pub fn frame_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Dummy{ buffer, .. } => buffer.as_mut_slice(),
            Self::WindowBacked(pixels) => pixels.frame_mut(),
        }
    }

    pub fn pixel_count(&self) -> usize {
        self.pixel_width() * self.pixel_height()
    }

    pub fn pixel_width(&self) -> usize {
        match self {
            Self::Dummy { pixel_width, .. } => *pixel_width,
            Self::WindowBacked(pixels) => pixels.texture().width() as usize,
        }
    }

    pub fn pixel_height(&self) -> usize {
        match self {
            Self::Dummy { pixel_height, .. } => *pixel_height,
            Self::WindowBacked(pixels) => pixels.texture().height() as usize,
        }
    }

    pub fn resize(&mut self, new_pixel_width: usize, new_pixel_height: usize) {
        match self {
            Self::Dummy { buffer, pixel_width, pixel_height } => {
                *buffer =  vec![0; 4 * new_pixel_width * new_pixel_height];
                *pixel_width = new_pixel_width;
                *pixel_height = new_pixel_height;
            }
            Self::WindowBacked(pixels) => pixels.resize_buffer(new_pixel_width as u32, new_pixel_height as u32).unwrap(),
        }
    }

    pub fn set_pixel(&mut self, column: usize, row: usize, rgb: Rgb) {
        assert!(column < self.pixel_width());
        assert!(row < self.pixel_height());

        let index = 4 * (self.pixel_width() * row + column);
        let buffer = self.frame_mut();
        buffer[index] = rgb.red();
        buffer[index + 1] = rgb.green();
        buffer[index + 2] = rgb.blue();
        // No transparency.
        buffer[index + 3] = 0xFF;
    }

    pub fn write_all_pixel_data(&self, data: &mut [u8]) {
        let (input_chunks, remainder): (&[[u8; 4]], &[u8]) = self.frame().as_chunks();
        assert!(remainder.is_empty());
        let (output_chunks, remainder): (&mut [[u8; 3]], &mut [u8]) = data.as_chunks_mut();
        assert!(remainder.is_empty());

        assert_eq!(input_chunks.len(), output_chunks.len());

        for ([ir, ig, ib, _ia], [or, og, ob]) in input_chunks.iter().zip(output_chunks) {
            *or = *ir;
            *og = *ig;
            *ob = *ib;
        }
    }

    pub fn to_ppm(&self) -> Ppm {
        let mut data = vec![0; 3 * self.pixel_count()];
        self.write_all_pixel_data(&mut data);
        Ppm::new(data, self.pixel_width(), self.pixel_height())
    }

    pub fn render_with<F>(&self, render_function: F) -> Result<(), pixels::Error>
    where
        F: FnOnce(
            &mut wgpu::CommandEncoder,
            &wgpu::TextureView,
            &PixelsContext,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        match self {
            Self::Dummy {..} => Ok(()),
            Self::WindowBacked(pixels) => pixels.render_with(render_function),
        }
    }

    pub fn max_texture_dimension_2d(&self) -> usize {
        match self {
            Self::Dummy {..} => 1_000_000,
            Self::WindowBacked(pixels) => pixels.device().limits().max_texture_dimension_2d as usize,
        }
    }

    pub fn wgpu_renderer(&self) -> Option<egui_wgpu::Renderer> {
        match self {
            Self::Dummy {..} => None,
            Self::WindowBacked(pixels) => {
                let renderer_options = egui_wgpu::RendererOptions::default();
                Some(egui_wgpu::Renderer::new(pixels.device(), pixels.render_texture_format(), renderer_options))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameBuffer<T> {
    buffer: Vec<T>,
    column_count: u16,
    row_count: u16,
}

impl<T: Copy> FrameBuffer<T> {
    pub fn filled(column_count: u16, row_count: u16, value: T) -> FrameBuffer<T> {
        Self {
            buffer: vec![value; (column_count * row_count) as usize],
            column_count,
            row_count,
        }
    }
}

impl<T: Default> FrameBuffer<T> {
    pub fn clear_row(&mut self, row: PixelRow) {
        for column in PixelColumn::iter() {
            self[PixelIndex { column, row }] = T::default();
        }
    }
}

impl<T: Copy + Default> FrameBuffer<T> {
    pub fn clear(&mut self) {
        *self = FrameBuffer::default();
    }
}

impl<T: Copy + Default> Default for FrameBuffer<T> {
    fn default() -> Self {
        Self::filled(PixelColumn::COLUMN_COUNT as u16, PixelRow::ROW_COUNT as u16, T::default())
    }
}

impl<T> Index<(u16, u16)> for FrameBuffer<T> {
    type Output = T;

    fn index(&self, (column, row): (u16, u16)) -> &T {
        assert!(column < self.column_count);
        assert!(row < self.row_count);
        &self.buffer[usize::from(row) * usize::from(self.column_count) + usize::from(column)]
    }
}

impl<T> IndexMut<(u16, u16)> for FrameBuffer<T> {
    fn index_mut(&mut self, (column, row): (u16, u16)) -> &mut T {
        assert!(column < self.column_count);
        assert!(row < self.row_count);
        &mut self.buffer[usize::from(row) * usize::from(self.column_count) + usize::from(column)]
    }
}

impl<T> Index<PixelIndex> for FrameBuffer<T> {
    type Output = T;

    fn index(&self, index: PixelIndex) -> &T {
        &self.buffer[index.to_usize()]
    }
}

impl<T> IndexMut<PixelIndex> for FrameBuffer<T> {
    fn index_mut(&mut self, index: PixelIndex) -> &mut T {
        &mut self.buffer[index.to_usize()]
    }
}

pub struct DebugBuffer<const WIDTH: usize, const HEIGHT: usize> {
    buffer: Box<[[Rgbt; WIDTH]; HEIGHT]>,
    background_rgb: Rgb,
}

impl<const WIDTH: usize, const HEIGHT: usize> DebugBuffer<WIDTH, HEIGHT> {
    pub fn new(background_rgb: Rgb) -> DebugBuffer<WIDTH, HEIGHT> {
        DebugBuffer {
            buffer: Box::new([[Rgbt::Transparent; WIDTH]; HEIGHT]),
            background_rgb,
        }
    }

    pub fn place_frame(&mut self, left_column: usize, top_row: usize, frame: &Frame) {
        assert_eq!(frame.pixel_count(), 4 * WIDTH * HEIGHT);
        let (chunks, remainder): (&[[u8; 4]], &[u8]) = frame.frame().as_chunks();
        assert!(remainder.is_empty());

        for (index, &[r, g, b, _a]) in chunks.iter().enumerate() {
            let rgb = Rgb::new(r, g, b);
            self.write(
                left_column + index % WIDTH,
                top_row + index / WIDTH,
                rgb,
            );
        }
    }

    pub fn place_frame_buffer_with<F, T: Copy>(
        &mut self,
        left_column: usize,
        top_row: usize,
        frame: &FrameBuffer<T>,
        transform: F,
    ) where F: Fn(T) -> Rgb {
        for index in PixelIndex::iter() {
            let value = frame[index];
            self.write(
                left_column + index.column.to_usize(),
                top_row + index.row.to_usize(),
                transform(value),
            );
        }
    }

    pub fn place_tile(&mut self, system_palette: &SystemPalette, left_column: usize, top_row: usize, tile: &Tile) {
        for row_in_tile in all::<RowInTile>() {
            for column_in_tile in all::<ColumnInTile>() {
                let column_in_tile = column_in_tile as usize;
                let row_in_tile = row_in_tile as usize;
                let pixel = system_palette.lookup_rgbt(tile.0[row_in_tile][column_in_tile], Emphasis::OFF);
                self.write_rgbt(
                    left_column + column_in_tile,
                    top_row + row_in_tile,
                    pixel,
                );
            }
        }
    }

    pub fn place_wrapping_horizontal_line(
        &mut self,
        row: usize,
        left_column: usize,
        right_column: usize,
        rgb: Rgb,
    ) {
        let row = row.rem_euclid(HEIGHT);
        let left_column = left_column.rem_euclid(WIDTH);
        let right_column = right_column.rem_euclid(WIDTH);
        if left_column < right_column {
            for column in left_column..=right_column {
                self.write(column, row, rgb);
            }
        } else {
            for column in left_column..WIDTH {
                self.write(column, row, rgb);
            }

            for column in 0..=right_column {
                self.write(column, row, rgb);
            }
        }
    }

    pub fn place_wrapping_vertical_line(
        &mut self,
        column: usize,
        top_row: usize,
        bottom_row: usize,
        rgb: Rgb,
    ) {
        let column = column.rem_euclid(WIDTH);
        let top_row = top_row.rem_euclid(HEIGHT);
        let bottom_row = bottom_row.rem_euclid(HEIGHT);
        if top_row < bottom_row {
            for row in top_row..=bottom_row {
                self.write(column, row, rgb);
            }
        } else {
            for row in top_row..HEIGHT {
                self.write(column, row, rgb);
            }

            for row in 0..=bottom_row {
                self.write(column, row, rgb);
            }
        }
    }

    pub fn copy_to_rgba_buffer(&self, buffer: &mut [u8]) {
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let index = 4 * (WIDTH * row + column);
                let pixel = self.read(column, row);
                buffer[index] = pixel.red();
                buffer[index + 1] = pixel.green();
                buffer[index + 2] = pixel.blue();
                // No transparency.
                buffer[index + 3] = 0xFF;
            }
        }
    }

    fn read(&self, column: usize, row: usize) -> Rgb {
        self.buffer[row][column]
            .to_rgb()
            .unwrap_or(self.background_rgb)
    }

    fn write(&mut self, column: usize, row: usize, rgb: Rgb) {
        self.buffer[row][column] = Rgbt::Opaque(rgb);
    }

    pub fn write_rgbt(&mut self, column: usize, row: usize, rgbt: Rgbt) {
        self.buffer[row][column] = rgbt;
    }
}