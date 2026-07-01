use std::ops::{Index, IndexMut};

use enum_iterator::all;

use crate::ppu::palette::color::Color;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::palette::composite_decoder::CompositeDecoder;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::gui::debug_screens::pattern_table::Tile;
use crate::ppu::pixel_index::{
    ColumnInTile, PixelColumn, PixelIndex, PixelRow, RowInTile,
};
use crate::ppu::register::ppu_registers::Emphasis;
use crate::ppu::render::ppm::Ppm;
use crate::ppu::sprite::sprite_attributes::Priority;

#[derive(Clone)]
pub struct Frame {
    buffer: FrameBuffer<(Color, Emphasis)>,

    background_buffer: FrameBuffer<ColorT>,
    sprite_buffer: FrameBuffer<(ColorT, Priority, bool)>,

    show_overscan: bool,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            buffer: FrameBuffer::filled((Color::BLACK, Emphasis::OFF)),

            background_buffer: FrameBuffer::filled(ColorT::Transparent),
            sprite_buffer: FrameBuffer::filled((ColorT::Transparent, Priority::Behind, false)),

            show_overscan: false,
        }
    }

    // Only used for debug windows.
    pub fn to_background_only(&self) -> Self {
        let mut frame = self.clone();
        frame.sprite_buffer = FrameBuffer::filled((ColorT::Transparent, Priority::Behind, false));
        frame
    }

    pub fn show_overscan_mut(&mut self) -> &mut bool {
        &mut self.show_overscan
    }

    pub fn set_pixel(&mut self, color: Color, emphasis: Emphasis, index: PixelIndex) {
        self.buffer[index] = (color, emphasis);
    }

    pub fn pixel(&self, index: PixelIndex) -> (Color, Emphasis, bool) {
        let visible = self.show_overscan || !index.is_in_overscan_region();
        let (color, emphasis) = self.buffer[index];
        (color, emphasis, visible)
    }

    #[inline]
    pub fn set_background_pixel(&mut self, index: PixelIndex, color: ColorT) {
        self.background_buffer[index] = color;
    }

    #[inline]
    pub fn set_sprite_pixel(
        &mut self,
        index: PixelIndex,
        color: ColorT,
        priority: Priority,
        is_sprite_0: bool,
    ) {
        self.sprite_buffer[index] = (color, priority, is_sprite_0);
    }

    pub fn write_all_pixel_data(&self, decoder: &dyn CompositeDecoder, data: &mut [u8]) {
        for pixel_index in PixelIndex::iter() {
            let (color, emphasis, _visible) = self.pixel(pixel_index);
            let pixel = decoder.decode_to_rgb(color, emphasis);

            let index = 3 * pixel_index.to_usize();
            data[index] = pixel.red();
            data[index + 1] = pixel.green();
            data[index + 2] = pixel.blue();
        }
    }

    pub fn copy_to_rgba_buffer(&self, decoder: &dyn CompositeDecoder, buffer: &mut [u8; 4 * PixelIndex::PIXEL_COUNT]) {
        for pixel_index in PixelIndex::iter() {
            let (color, emphasis, visible) = self.pixel(pixel_index);
            let mut pixel = decoder.decode_to_rgb(color, emphasis);
            if !visible {
                pixel = Rgb::BLACK;
            }

            let index = 4 * pixel_index.to_usize();
            buffer[index] = pixel.red();
            buffer[index + 1] = pixel.green();
            buffer[index + 2] = pixel.blue();
            // No transparency.
            buffer[index + 3] = 0xFF;
        }
    }

    pub fn to_ppm(&self, decoder: &dyn CompositeDecoder) -> Ppm {
        let mut data = vec![0; 3 * PixelIndex::PIXEL_COUNT];
        self.write_all_pixel_data(decoder, &mut data);
        Ppm::new(data)
    }
}

// Debug window methods.
impl Frame {
    // Used for debug windows only
    pub fn clear(&mut self) {
        // FIXME: Don't allocate new FrameBuffers to do this.
        self.buffer = FrameBuffer::filled((Color::BLACK, Emphasis::OFF));
        self.background_buffer = FrameBuffer::filled(ColorT::Transparent);
        self.sprite_buffer = FrameBuffer::filled((ColorT::Transparent, Priority::Behind, false));
    }

    pub fn clear_sprite_line(&mut self, row: PixelRow) {
        for column in PixelColumn::iter() {
            self.sprite_buffer[PixelIndex { column, row }] = (ColorT::Transparent, Priority::Behind, false);
        }
    }
}

#[derive(Clone)]
struct FrameBuffer<T>(Box<[T; PixelColumn::COLUMN_COUNT * PixelRow::ROW_COUNT]>);

impl<T: Copy> FrameBuffer<T> {
    fn filled(value: T) -> FrameBuffer<T> {
        FrameBuffer(Box::new([value; PixelColumn::COLUMN_COUNT * PixelRow::ROW_COUNT]))
    }
}

impl<T> Index<PixelIndex> for FrameBuffer<T> {
    type Output = T;

    fn index(&self, index: PixelIndex) -> &T {
        &self.0[index.to_usize()]
    }
}

impl<T> IndexMut<PixelIndex> for FrameBuffer<T> {
    fn index_mut(&mut self, index: PixelIndex) -> &mut T {
        &mut self.0[index.to_usize()]
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

    pub fn place_frame(&mut self, decoder: &dyn CompositeDecoder, left_column: usize, top_row: usize, frame: &Frame) {
        for index in PixelIndex::iter() {
            let (color, emphasis, _visible) = frame.pixel(index);
            let pixel = decoder.decode_to_rgb(color, emphasis);
            self.write(
                left_column + index.column.to_usize(),
                top_row + index.row.to_usize(),
                pixel,
            );
        }
    }

    pub fn place_tile(&mut self, decoder: &dyn CompositeDecoder, left_column: usize, top_row: usize, tile: &Tile) {
        for row_in_tile in all::<RowInTile>() {
            for column_in_tile in all::<ColumnInTile>() {
                let column_in_tile = column_in_tile as usize;
                let row_in_tile = row_in_tile as usize;
                let pixel = decoder.decode_to_rgbt(tile.0[row_in_tile][column_in_tile], Emphasis::OFF);
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
