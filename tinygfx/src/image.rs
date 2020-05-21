use super::color::Color;
use super::{Clip, Renderer};

pub trait MonoImageData {
    fn render_transparent<ColorType: Color>(
        &self,
        row: &mut Renderer<ColorType>,
        clip: Clip,
        x: i32,
        y: i32,
        color: ColorType,
    );

    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

pub struct MonoBitmapImage {
    pub data: &'static [u8],
    pub width: u16,
    pub height: u16,
    pub stride: u16,
}

impl MonoImageData for MonoBitmapImage {
    fn render_transparent<ColorType: Color>(
        &self,
        renderer: &mut Renderer<ColorType>,
        clip: Clip,
        x: i32,
        y: i32,
        color: ColorType,
    ) {
        for row in 0..self.height as i32 {
            if row + y < clip.top() {
                continue;
            }
            if row + y >= clip.bottom() {
                break;
            }
            let row_index = (row as i32 * self.stride as i32) as usize;
            // TODO: Not transparent.
            renderer.render_bitmap_row(
                clip,
                y + row,
                x,
                x + self.width as i32,
                &self.data[row_index..row_index + self.stride as usize],
                color,
            );
        }
    }

    fn width(&self) -> u32 {
        self.width as u32
    }
    fn height(&self) -> u32 {
        self.height as u32
    }
}

pub struct MonoRLEImage {
    pub data: &'static [u16],
    pub width: u16,
    pub height: u16,
}

impl MonoImageData for MonoRLEImage {
    // TODO: Naming in the whole library: row vs renderer
    fn render_transparent<ColorType: Color>(
        &self,
        renderer: &mut Renderer<ColorType>,
        clip: Clip,
        x: i32,
        y: i32,
        color: ColorType,
    ) {
        for row in 0..self.height as i32 {
            if row + y < clip.top() {
                continue;
            }
            if row + y >= clip.bottom() {
                break;
            }
            let line_start = self.data[row as usize] as usize;
            let line_end = self.data[row as usize + 1] as usize;
            let line = &self.data[line_start..line_end];

            renderer.render_rle_row(clip, x, row + y, line, color);
        }
    }

    fn width(&self) -> u32 {
        self.width as u32
    }
    fn height(&self) -> u32 {
        self.height as u32
    }
}
