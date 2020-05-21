#![no_std]

pub mod color;
pub mod font;
pub mod image;

use core::cmp::max;
use core::cmp::min;
use core::marker::PhantomData;

use color::Color;
use font::Font;
use image::MonoImageData;

pub struct Renderer<'a, ColorType> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
    current_top: i32,
    current_bottom: i32,
    mirror_y: bool,
    phantom: PhantomData<ColorType>,
}

impl<'a, ColorType> Renderer<'a, ColorType>
where
    ColorType: Color,
{
    pub fn fill(
        &mut self,
        clip: Clip,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        color: ColorType,
    ) {
        let clip = clip
            .clip(left, top, right, bottom)
            .clip_top(self.current_top)
            .clip_bottom(self.current_bottom);
        if clip.is_empty() {
            return;
        }
        for y in clip.top()..clip.bottom() {
            color.fill(self.row_buffer(y), clip.left(), clip.right());
        }
    }

    pub fn render_rle_row(&mut self, clip: Clip, x: i32, y: i32, data: &[u16], color: ColorType) {
        let clip = clip
            .clip_top(self.current_top)
            .clip_bottom(self.current_bottom);
        if !clip.contains_row(y) {
            return;
        }
        let row_buffer = self.row_buffer(y);
        let mut pos = x;
        for run in data {
            let length = (run & 0x7fff) as u32;
            if (run >> 15) != 0u16 {
                let run_clip = clip.clip_left(pos).clip_right(pos + length as i32);
                color.fill(row_buffer, run_clip.left(), run_clip.right());
            }
            pos += length as i32;
        }
    }

    pub fn render_bitmap_row(
        &mut self,
        clip: Clip,
        y: i32,
        left: i32,
        right: i32,
        bits: &[u8],
        color: ColorType,
    ) {
        // TODO: Color!
        let clip = clip.clip(left, self.current_top, right, self.current_bottom);
        if clip.is_empty() || !clip.contains_row(y) {
            return;
        }

        let row_buffer = self.row_buffer(y);
        color.render_bitmap_row(
            row_buffer,
            left,
            bits,
            clip.left() - left,
            clip.right() - right,
        );
    }

    pub fn clear(&mut self, color: ColorType) {
        self.fill(
            self.full_frame(),
            0,
            0,
            self.width as i32,
            self.height as i32,
            color,
        );
    }

    pub fn full_frame(&self) -> Clip {
        Clip {
            left: 0,
            right: self.width as i32,
            top: self.current_top,
            bottom: self.current_bottom,
        }
    }

    pub fn current_top_row(&self) -> i32 {
        self.current_top
    }

    pub fn current_bottom_row(&self) -> i32 {
        self.current_bottom
    }

    fn row_buffer(&mut self, row: i32) -> &mut [u8] {
        let y_offset = if self.mirror_y {
            self.current_bottom - 1 - row
        } else {
            row - self.current_top
        } as usize;
        let stride = (self.width as usize * ColorType::bits_per_pixel() + 7) >> 3;
        &mut self.buffer[y_offset * stride..(y_offset + 1) * stride]
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

    pub fn contains_row(self, row: i32) -> bool {
        if (row) < self.top {
            false
        } else if row >= self.bottom {
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

    // TODO: top/bottom instead of y?
    pub fn draw_part(&self, y: u32, buffer: &mut [u8]) {
        let stride = (self.width as usize * ColorType::bits_per_pixel() + 7) >> 3;
        let lines = buffer.len() / stride;
        //assert!(buffer.len() * 8 >= self.width as usize * ColorType::bits_per_pixel()); TODO
        let mut top = y as i32;
        let mut bottom = y as i32 + lines as i32;
        if self.mirror_y {
            top = self.height as i32 - bottom;
            bottom = self.height as i32 - y as i32;
        }
        (self.draw)(Renderer {
            buffer,
            width: self.width,
            height: self.height,
            current_top: top,
            current_bottom: bottom,
            mirror_y: self.mirror_y,
            phantom: PhantomData,
        });
        if self.mirror_x {
            for i in 0..lines {
                ColorType::mirror_x(
                    &mut buffer[i * stride..(i + 1) * stride],
                    self.width as usize,
                );
            }
        }
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
        renderer.fill(
            clip,
            self.left,
            self.top,
            self.left + self.width,
            self.top + self.height,
            self.color,
        );
    }
}

pub struct Text<'a, ColorType, FontImage, StringType> {
    text: StringType,
    x: i32,
    y: i32,
    x_align: i32,
    font: &'a Font<'a, FontImage>,
    color: ColorType,
}

impl<'a, ColorType, FontImage, StringType> Text<'a, ColorType, FontImage, StringType>
where
    ColorType: Color,
    FontImage: MonoImageData,
    StringType: AsRef<str>,
{
    pub fn new(
        x: i32,
        y: i32,
        text: StringType,
        font: &'a Font<FontImage>,
        color: ColorType,
    ) -> Self {
        Self {
            text,
            x,
            y,
            x_align: 0,
            font,
            color,
        }
    }

    pub fn align(&mut self, align: TextAlignment) {
        self.x_align = match align {
            TextAlignment::Left => 0,
            TextAlignment::Right => -(self.font.get_text_size(self.text.as_ref()).0 as i32),
            TextAlignment::Center => -(self.font.get_text_size(self.text.as_ref()).0 as i32) / 2,
        };
    }

    pub fn draw(&self, clip: Clip, renderer: &mut Renderer<ColorType>) {
        self.font.render(
            renderer,
            clip,
            self.text.as_ref(),
            self.x + self.x_align,
            self.y,
            self.color,
        );
    }
}

pub struct MonoImage<'a, ImageType, ColorType> {
    image: &'a ImageType,
    x: i32,
    y: i32,
    color: ColorType,
}

impl<'a, ImageType, ColorType> MonoImage<'a, ImageType, ColorType>
where
    ImageType: MonoImageData,
    ColorType: Color,
{
    pub fn new(x: i32, y: i32, image: &'a ImageType, color: ColorType) -> Self {
        Self { image, x, y, color }
    }

    pub fn draw(&self, clip: Clip, renderer: &mut Renderer<ColorType>) {
        self.image
            .render_transparent(renderer, clip, self.x, self.y, self.color);
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TextAlignment {
    Left,
    Right,
    Center,
}

#[cfg(test)]
mod tests {
    use super::color::BlackWhite::{self, Black, White};
    use super::{Frame, Renderer};

    /*#[test]
    #[should_panic]
    fn test_row_renderer_new_panic() {
        let mut buffer = [0u8];
        Frame::new(12, 1, |renderer: Renderer<BlackWhite>| {}).draw_row(0, &mut buffer);
    }*/

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
                renderer.fill(clip, test.fill.0, 0, test.fill.1, 1, test.color);
            })
            .draw_part(0, &mut buffer);
            assert_eq!(buffer, test.ok);
        }
    }
}
