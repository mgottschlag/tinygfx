#![no_std]

pub mod color;
pub mod font;
pub mod image;

use core::cmp::max;
use core::cmp::min;
use core::marker::PhantomData;

use color::Color;

pub struct Renderer<'a, ColorType> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
    row: u32,
    mirror_x: bool,
    phantom: PhantomData<ColorType>,
}

impl<'a, ColorType> Renderer<'a, ColorType>
where
    ColorType: Color,
{
    pub fn fill(&mut self, clip: Clip, left: i32, right: i32, color: ColorType) {
        // TODO: y clipping!
        // TODO: self.mirror_x
        let line_clip = clip.clip_left(left).clip_right(right);
        if line_clip.is_empty() {
            return;
        }
        color.fill(self.buffer, line_clip.left(), line_clip.right());
    }

    pub fn render_bitmap(
        &mut self,
        clip: Clip,
        left: i32,
        right: i32,
        bits: &[u8],
        color: ColorType,
    ) {
        // TODO: self.mirror_x
        // TODO: Color!
        // TODO: y clipping!
        let line_clip = clip.clip_left(left).clip_right(right);
        if line_clip.is_empty() {
            return;
        }

        for x in line_clip.left()..line_clip.right() {
            let byte = (x - left) / 8;
            let bit = (x - left) & 7;
            if (bits[byte as usize] & (1 << bit)) != 0 {
                self.buffer[(x / 8) as usize] |= 0x80 >> (x & 7);
            } else {
                self.buffer[(x / 8) as usize] &= !(0x80 >> (x & 7));
            }
        }
    }

    pub fn clear(&mut self, color: ColorType) {
        self.fill(self.full_frame(), 0, self.width as i32, color);
    }

    pub fn full_frame(&self) -> Clip {
        Clip {
            left: 0,
            right: self.width as i32,
            top: 0,
            bottom: self.height as i32,
        }
    }

    pub fn current_row(&self) -> u32 {
        self.row
    }
}

#[derive(Copy, Clone)]
pub struct Clip {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Clip {
    pub fn left(&self) -> i32 {
        self.left
    }

    pub fn right(&self) -> i32 {
        self.right
    }

    pub fn top(&self) -> i32 {
        self.top
    }

    pub fn bottom(&self) -> i32 {
        self.bottom
    }

    pub fn clip_left(self, left: i32) -> Clip {
        Clip {
            left: max(self.left, left),
            top: self.top,
            right: self.right,
            bottom: self.bottom,
        }
    }

    pub fn clip_right(self, right: i32) -> Clip {
        Clip {
            left: self.left,
            top: self.top,
            right: min(self.right, right),
            bottom: self.bottom,
        }
    }

    pub fn clip_top(self, top: i32) -> Clip {
        Clip {
            left: self.left,
            top: max(self.top, top),
            right: self.right,
            bottom: self.bottom,
        }
    }

    pub fn clip_bottom(self, bottom: i32) -> Clip {
        Clip {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: min(self.bottom, bottom),
        }
    }

    pub fn clip(self, left: i32, top: i32, right: i32, bottom: i32) -> Clip {
        Clip {
            left: max(self.left, left),
            top: max(self.top, top),
            right: min(self.right, right),
            bottom: min(self.bottom, bottom),
        }
    }

    pub fn contains_row(self, row: u32) -> bool {
        if (row as i32) < self.top {
            false
        } else if row as i32 >= self.bottom {
            false
        } else {
            true
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
    Draw: Fn(Renderer<ColorType>),
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
        assert!(buffer.len() * 8 >= self.width as usize * ColorType::bits_per_pixel());
        (self.draw)(Renderer {
            buffer,
            width: self.width,
            height: self.height,
            row: y,
            mirror_x: self.mirror_x,
            phantom: PhantomData,
        });
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn width(&self) -> u32 {
        self.width
    }
}

pub struct Rectangle<ColorType> {
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    color: ColorType,
}

impl<ColorType> Rectangle<ColorType>
where
    ColorType: Color,
{
    pub fn new(left: i32, top: i32, width: i32, height: i32, color: ColorType) -> Self {
        Self {
            left,
            top,
            width,
            height,
            color,
        }
    }

    pub fn draw(&self, clip: Clip, renderer: &mut Renderer<ColorType>) {
        if !clip
            .clip_top(self.top)
            .clip_bottom(self.top + self.height)
            .contains_row(renderer.current_row())
        {
            return;
        }
        renderer.fill(clip, self.left, self.left + self.width, self.color);
    }
}

#[cfg(test)]
mod tests {
    use super::color::BlackWhite::{self, Black, White};
    use super::{Frame, Renderer};

    #[test]
    #[should_panic]
    fn test_row_renderer_new_panic() {
        let mut buffer = [0u8];
        Frame::new(12, 1, |renderer: Renderer<BlackWhite>| {}).draw_row(0, &mut buffer);
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
        // TODO: Tests for y clipping.
        for test in &tests {
            let mut buffer = test.before;
            Frame::new(32, 1, |mut renderer| {
                let clip = renderer.full_frame();
                let clip = clip.clip_left(test.clip.0).clip_right(test.clip.1);
                renderer.fill(clip, test.fill.0, test.fill.1, test.color);
            })
            .draw_row(0, &mut buffer);
            assert_eq!(buffer, test.ok);
        }
    }
}
