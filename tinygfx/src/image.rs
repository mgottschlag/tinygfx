use super::color::Color;
use super::{Clip, Renderer};

pub struct MonoBitmapImage {
    pub data: &'static [u8],
    pub width: u16,
    pub height: u16,
    pub stride: u16,
}

impl MonoBitmapImage {
    pub fn render_row_transparent<ColorType: Color>(
        &self,
        row: &mut Renderer<ColorType>,
        clip: Clip,
        y: i32,
        offset: i32,
        color: ColorType,
    ) {
        if y < 0 || y >= self.height as i32 {
            return;
        }
        let row_index = (y * self.stride as i32) as usize;
        row.render_bitmap(
            clip,
            offset,
            offset + self.width as i32,
            &self.data[row_index..row_index + self.stride as usize],
            color,
        );
    }
}

pub struct MonoRLEImage {
    pub data: &'static [u16],
    pub width: u16,
    pub height: u16,
}

impl MonoRLEImage {
    // TODO: Naming in the whole library: row vs renderer
    pub fn render_row_transparent<ColorType: Color>(
        &self,
        row: &mut Renderer<ColorType>,
        clip: Clip,
        y: i32,
        offset: i32,
        color: ColorType,
    ) {
        if y < 0 {
            return;
        }
        if y >= self.height as i32 {
            return;
        }
        let line_start = self.data[y as usize] as usize;
        let line_end = self.data[y as usize + 1] as usize;
        let line = &self.data[line_start..line_end];

        let mut pos = 0;
        for run in line {
            let length = (run & 0x7fff) as u32;
            if (run >> 15) != 0u16 {
                row.fill(clip, offset + pos, offset + pos + length as i32, color);
            }
            pos += length as i32;
        }
    }
}
