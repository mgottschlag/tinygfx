#![no_std]

pub mod color;
pub mod font;
pub mod image;

use core::cmp::max;
use core::cmp::min;
use core::marker::PhantomData;

use color::Color;

pub struct RowRenderer<'a, ColorType> {
    buffer: &'a mut [u8],
    width: u32,
    mirror_x: bool,
    phantom: PhantomData<ColorType>,
}

impl<'a, ColorType> RowRenderer<'a, ColorType>
where
    ColorType: Color,
{
    fn new(buffer: &'a mut [u8], width: u32, mirror_x: bool) -> Self {
        assert!(buffer.len() * 8 >= width as usize * ColorType::bits_per_pixel());
        RowRenderer {
            buffer,
            width,
            mirror_x,
            phantom: PhantomData,
        }
    }

    pub fn fill(&mut self, clip: &ClipRow, left: i32, right: i32, color: ColorType) {
        // TODO: self.mirror_x
        let line_clip = clip.clip(left, right);
        if line_clip.is_empty() {
            return;
        }
        let (left, right) = line_clip.get();
        color.fill(self.buffer, left, right);
    }

    pub fn render_bitmap(
        &mut self,
        clip: &ClipRow,
        left: i32,
        right: i32,
        bits: &[u8],
        color: ColorType,
    ) {
        // TODO: self.mirror_x
        // TODO: Color!
        let line_clip = clip.clip(left, right);
        if line_clip.is_empty() {
            return;
        }

        for x in line_clip.get().0..line_clip.get().1 {
            let byte = (x - left) / 8;
            let bit = (x - left) & 7;
            if (bits[byte as usize] & (1 << bit)) != 0 {
                self.buffer[(x / 8) as usize] |= 0x80 >> (x & 7);
            } else {
                self.buffer[(x / 8) as usize] &= !(0x80 >> (x & 7));
            }
        }
    }

    pub fn full_row(&self) -> ClipRow {
        ClipRow {
            left: 0,
            right: self.width as i32,
        }
    }
}

pub struct ClipRow {
    left: i32,
    right: i32,
}

impl ClipRow {
    pub fn get(&self) -> (i32, i32) {
        (self.left, self.right)
    }

    pub fn clip(&self, left: i32, right: i32) -> ClipRow {
        ClipRow {
            left: max(self.left, left),
            right: min(self.right, right),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.left >= self.right
    }
}

pub struct Frame<Draw, ColorType> {
    width: u32,
    height: u32,
    draw: Draw,
    mirror_x: bool,
    mirror_y: bool,
    phantom: PhantomData<ColorType>,
}

impl<Draw, ColorType> Frame<Draw, ColorType>
where
    Draw: Fn(u32, RowRenderer<ColorType>),
    ColorType: Color,
{
    pub fn new(width: u32, height: u32, draw: Draw) -> Self {
        Self {
            width,
            height,
            draw,
            mirror_x: false,
            mirror_y: false,
            phantom: PhantomData::<ColorType>,
        }
    }

    pub fn mirror_x(&mut self, mirror_x: bool) {
        self.mirror_x = mirror_x;
    }

    pub fn mirror_y(&mut self, mirror_y: bool) {
        self.mirror_y = mirror_y;
    }

    pub fn draw_row(&self, mut y: u32, buffer: &mut [u8]) {
        if self.mirror_y {
            y = self.height - y - 1;
        }
        let renderer = RowRenderer::new(buffer, self.width, self.mirror_x);
        (self.draw)(y, renderer);
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }
}

#[cfg(test)]
mod tests {
    use super::color::BlackWhite::{self, Black, White};
    use super::RowRenderer;

    #[test]
    #[should_panic]
    fn test_row_renderer_new_panic() {
        let mut buffer = [0u8];
        RowRenderer::<BlackWhite>::new(&mut buffer, 12, false);
    }

    struct FillTest {
        before: [u8; 4],
        clip: (i32, i32),
        fill: (i32, i32),
        color: BlackWhite,
        ok: [u8; 4],
    }

    #[test]
    fn test_row_renderer_fill() {
        let tests = [
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (4, 16),
                color: White,
                ok: [0x0f, 0xff, 0x0, 0x0],
            },
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (4, 20),
                color: White,
                ok: [0x0f, 0xff, 0xf0, 0x0],
            },
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (2, 6),
                color: White,
                ok: [0x3c, 0x0, 0x0, 0x0],
            },
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (2, 6),
                color: Black,
                ok: [0x0, 0x0, 0x0, 0x0],
            },
            FillTest {
                before: [0xff; 4],
                clip: (0, 32),
                fill: (2, 6),
                color: Black,
                ok: [0xc3, 0xff, 0xff, 0xff],
            },
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (5, 5),
                color: White,
                ok: [0; 4],
            },
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (-5, -4),
                color: White,
                ok: [0; 4],
            },
            FillTest {
                before: [0; 4],
                clip: (9, 32),
                fill: (4, 12),
                color: White,
                ok: [0, 0x70, 0, 0],
            },
            FillTest {
                before: [0; 4],
                clip: (6, 5),
                fill: (4, 12),
                color: White,
                ok: [0, 0, 0, 0],
            },
            FillTest {
                before: [0; 4],
                clip: (0, 32),
                fill: (16, 32),
                color: White,
                ok: [0, 0, 0xff, 0xff],
            },
        ];
        for test in &tests {
            let mut buffer = test.before;
            let mut renderer = RowRenderer::new(&mut buffer[..], 32, false);
            let clip = renderer.full_row();
            let clip = clip.clip(test.clip.0, test.clip.1);
            renderer.fill(&clip, test.fill.0, test.fill.1, test.color);
            assert_eq!(buffer, test.ok);
        }
    }
}
