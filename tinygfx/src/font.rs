use super::color::Color;
use super::image::MonoImageData;
use super::{Clip, Renderer};

pub struct Font<'a, Image> {
    pub ascender: u16,
    pub descender: u16,
    pub glyphs: &'a [Glyph<Image>],
    pub get_glyph_index: fn(c: char) -> Option<usize>,
}

impl<'a, Image> Font<'a, Image>
where
    Image: MonoImageData,
{
    pub fn get_text_size(&self, text: &str) -> (u32, u32) {
        let mut width = 0;
        for c in text.chars() {
            let index = (self.get_glyph_index)(c);
            if index.is_none() {
                continue;
            }
            let glyph = &self.glyphs[index.unwrap()];
            width += glyph.advance;
        }
        (width, (self.ascender + self.descender) as u32)
    }

    pub fn render<ColorType: Color>(
        &self,
        renderer: &mut Renderer<ColorType>,
        clip: Clip,
        text: &str,
        x: i32,
        y: i32,
        color: ColorType,
    ) {
        let mut pos = x;
        // TODO: Discard glyphs if they are definitely not shown.
        for c in text.chars() {
            let index = (self.get_glyph_index)(c);
            if index.is_none() {
                continue;
            }
            let glyph = &self.glyphs[index.unwrap()];
            self.render_glyph(renderer, clip, glyph, pos, y, color);
            pos += glyph.advance as i32;
        }
        //}
    }

    fn render_glyph<ColorType: Color>(
        &self,
        renderer: &mut Renderer<ColorType>,
        clip: Clip,
        glyph: &Glyph<Image>,
        x: i32,
        y: i32,
        color: ColorType,
    ) {
        let x = x + glyph.image_left as i32;
        let y = y + self.ascender as i32 - glyph.image_top as i32;
        glyph.image.render_transparent(renderer, clip, x, y, color);
    }
}

pub struct Glyph<Image> {
    pub image: Image,
    pub image_left: i16,
    pub image_top: i16,
    pub advance: u32,
}
