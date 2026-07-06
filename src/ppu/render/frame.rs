use std::ops::{Index, IndexMut};

use enum_iterator::all;

use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::gui::debug_screens::pattern_table::Tile;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pixel_index::{
    ColumnInTile, PixelColumn, PixelIndex, PixelRow, RowInTile,
};
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::ppm::Ppm;

const STANDARD_WIDTH: u16 = PixelColumn::COLUMN_COUNT as u16;
const STANDARD_HEIGHT: u16 = PixelRow::ROW_COUNT as u16;

#[derive(Clone, Debug)]
pub struct Frame {
    buffer: FrameBuffer<Rgb>,
}

impl Frame {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: FrameBuffer::filled(width, height, Rgb::BLACK),
        }
    }

    pub fn exact_sized() -> Self {
        Self::new(STANDARD_WIDTH, STANDARD_HEIGHT)
    }

    pub fn width(&self) -> u16 {
        self.buffer.column_count
    }

    pub fn height(&self) -> u16 {
        self.buffer.row_count
    }

    pub fn set_pixel(&mut self, pixel_index: PixelIndex, rgb: Rgb) {
        self.buffer[pixel_index] = rgb;
    }

    pub fn pixel(&self, index: PixelIndex) -> Rgb {
        self.buffer[index]
    }

    pub fn write_all_pixel_data(&self, data: &mut [u8]) {
        for pixel_index in PixelIndex::iter() {
            let rgb = self.pixel(pixel_index);

            let index = 3 * pixel_index.to_usize();
            data[index] = rgb.red();
            data[index + 1] = rgb.green();
            data[index + 2] = rgb.blue();
        }
    }

    pub fn copy_to_rgba_buffer(&self, buffer: &mut [u8; 4 * PixelIndex::PIXEL_COUNT], show_overscan: bool) {
        for pixel_index in PixelIndex::iter() {
            let mut rgb = self.pixel(pixel_index);
            if show_overscan && pixel_index.is_in_overscan_region() {
                // TODO: Probably make these pixels transparent instead.
                rgb = Rgb::BLACK;
            }

            let index = 4 * pixel_index.to_usize();
            buffer[index] = rgb.red();
            buffer[index + 1] = rgb.green();
            buffer[index + 2] = rgb.blue();
            // No transparency.
            buffer[index + 3] = 0xFF;
        }
    }

    pub fn to_ppm(&self) -> Ppm {
        let mut data = vec![0; 3 * PixelIndex::PIXEL_COUNT];
        self.write_all_pixel_data(&mut data);
        Ppm::new(data)
    }
}

// Debug window methods.
impl Frame {
    // Used for debug windows only
    pub fn clear(&mut self) {
        // FIXME: Don't allocate new FrameBuffers to do this.
        self.buffer = FrameBuffer::filled(self.buffer.column_count, self.buffer.row_count, Rgb::BLACK);
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
        for index in PixelIndex::iter() {
            let rgb = frame.pixel(index);
            self.write(
                left_column + index.column.to_usize(),
                top_row + index.row.to_usize(),
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