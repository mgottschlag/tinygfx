use core::convert::TryInto;

pub trait Color: Copy + Clone {
    fn bits_per_pixel() -> usize;
    fn fill(&self, buffer: &mut [u8], left: i32, right: i32);
    fn render_bitmap_row(&self, buffer: &mut [u8], x: i32, bitmap: &[u8], left: i32, right: i32);
    fn mirror_x(buffer: &mut [u8], width: usize);
}

#[derive(Copy, Clone)]
pub enum BlackWhite {
    White,
    Black,
}

impl Color for BlackWhite {
    fn bits_per_pixel() -> usize {
        1
    }

    fn fill(&self, buffer: &mut [u8], left: i32, right: i32) {
        let (left_index, right_index) = ((left >> 3) as usize, (right >> 3) as usize);
        let (left_offset, right_offset) = (left & 7, right & 7);
        let left_mask = 0xffu8 >> left_offset;
        let right_mask = (0xff00u16 >> right_offset) as u8;

        if left_index == right_index {
            // Both ends are in the same byte.
            let mask = left_mask & right_mask;
            if let Self::White = self {
                buffer[left_index] |= mask;
            } else {
                buffer[left_index] &= !mask;
            }
        } else {
            // We cross byte boundaries.
            if let Self::White = self {
                buffer[left_index] |= left_mask;
                for i in (left_index + 1)..right_index {
                    buffer[i] = 0xff;
                }
                if right_offset != 0 {
                    buffer[right_index] |= right_mask;
                }
            } else {
                buffer[left_index] &= !left_mask;
                for i in (left_index + 1)..right_index {
                    buffer[i] = 0x0;
                }
                if right_offset != 0 {
                    buffer[right_index] &= !right_mask;
                }
            }
        }
    }

    fn render_bitmap_row(
        &self,
        buffer: &mut [u8],
        x: i32,
        bitmap: &[u8],
        mut left: i32,
        right: i32,
    ) {
        //let left_mask = 0xff >> (left & !7);
        //let right_mask = (0xff00u16 >> (right & !7)) as u8;

        if (left & !7) == (right & !7) {
            // Only one byte to copy.
            self.render_bitmap_row_naive(buffer, x, bitmap, left, right);
            // TODO: Probably, the naive method is cheaper!
            /*let mask = left_mask & right_mask;
            let value = bitmap[left as usize & !7] & mask;
            let left_mask = 0xff >> output_shift;
            let left_value = value >> output_shift;
            let right_mask = 0xff << (8 - output_shift);
            let right_value = value << (8 - output_shift);
            buffer[first_byte] = (buffer[first_byte] & left_mask) | left_value;
            if first_byte != buffer.len() {
                buffer[first_byte + 1] = (buffer[first_byte + 1] & right_mask) | right_value;
            }*/
            return;
        }

        let output_shift = x & 0x7;
        if left & 7 != 0 {
            // Write the first (partial) byte.
            let next_left = (left + 7) & !7;
            self.render_bitmap_row_naive(buffer, x, bitmap, left, next_left);
            left = next_left;
        }

        if output_shift != 0 {
            while left + 8 <= right {
                let index = (x + left) as usize & !7;
                let value = bitmap[left as usize >> 3];
                let left_mask = 0xff >> output_shift;
                let left_value = value >> output_shift;
                let right_mask = 0xff << (8 - output_shift);
                let right_value = value << (8 - output_shift);
                buffer[index] = (buffer[index] & left_mask) | left_value;
                buffer[index + 1] = (buffer[index + 1] & right_mask) | right_value;
                left += 8;
            }
        } else {
            // Aligned => direct copy.
            let next_left = right & !7;
            let first_output_index = (x + left) as usize >> 3;
            let last_output_index = (x + next_left) as usize >> 3;
            let first_input_index = left as usize >> 3;
            let last_input_index = next_left as usize >> 3;
            buffer[first_output_index..last_output_index]
                .copy_from_slice(&bitmap[first_input_index..last_input_index]);
            left = next_left;
        }

        if left != right {
            // Write the last (partial) byte.
            self.render_bitmap_row_naive(buffer, x, bitmap, left, right);
        }
    }

    fn mirror_x(buffer: &mut [u8], width: usize) {
        let mut left = 0;
        let mut right = (width + 7) >> 3;
        // 4 bytes at a time.
        while right - 8 >= left {
            let left_value = u32::from_ne_bytes(buffer[left..left + 4].try_into().unwrap());
            let right_value = u32::from_ne_bytes(buffer[right - 4..right].try_into().unwrap());
            let left_value = left_value.reverse_bits();
            let right_value = right_value.reverse_bits();
            buffer[left..left + 4].copy_from_slice(&right_value.to_ne_bytes());
            buffer[right - 4..right].copy_from_slice(&left_value.to_ne_bytes());
            right -= 4;
            left += 4;
        }
        if right - 4 == left {
            let value = u32::from_ne_bytes(buffer[right - 4..right].try_into().unwrap());
            buffer[right - 4..right].copy_from_slice(&value.reverse_bits().to_ne_bytes());
            right -= 4;
            if (width & 0x7) == 0 {
                return;
            }
        }

        // 1 byte at a time.
        while right - 2 >= left {
            let left_value = buffer[left];
            let right_value = buffer[right - 1];
            let left_value = left_value.reverse_bits();
            let right_value = right_value.reverse_bits();
            buffer[left] = right_value;
            buffer[right - 1] = left_value;
            right -= 4;
            left += 4;
        }
        if right - 1 == left {
            buffer[left] = buffer[left].reverse_bits();
        }

        // If the first byte is now incomplete, shift to the left.
        let remainder = width & 0x7;
        if remainder != 0 {
            let bytes = (width + 7) >> 3;
            let shift = 8 - remainder;
            for i in 0..(bytes - 1) {
                buffer[i] = (buffer[i] << shift) | (buffer[i + 1] >> remainder);
            }
            buffer[bytes - 1] <<= shift;
        }
    }
}

impl BlackWhite {
    fn render_bitmap_row_naive(
        &self,
        buffer: &mut [u8],
        x: i32,
        bitmap: &[u8],
        left: i32,
        right: i32,
    ) {
        for x in left + x..right + x {
            let byte = (x - left) >> 3;
            let bit = (x - left) & 7;
            if (bitmap[byte as usize] & (1 << bit)) != 0 {
                buffer[(x / 8) as usize] |= 0x80 >> (x & 7);
            } else {
                buffer[(x / 8) as usize] &= !(0x80 >> (x & 7));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BlackWhite::{self, Black, White};
    use super::Color;

    #[test]
    fn bitmap_test() {
        // (color, bitmap, x, left, right, expected)
        let tests = [
            (White, 3, &[0], 3, 4, &[0; 4]),
            (White, 3, &[0xff], 3, 4, &[0x02, 0x00, 0x00, 0x00]),
            (White, 3, &[0xff], 0, 8, &[0x1f, 0xe0, 0x00, 0x00]),
            (White, 0, &[0xff], 0, 8, &[0xff, 0x00, 0x00, 0x00]),
            (White, 0, &[0xff], 3, 7, &[0x1e, 0x00, 0x00, 0x00]),
        ];
        for test in tests.iter() {
            let mut output = [0; 4];
            test.0
                .render_bitmap_row(&mut output, test.1, test.2, test.3, test.4);
            assert_eq!(&output, test.5);
        }
        // TODO
    }
}

/*#[derive(Copy, Clone)]
pub enum BlackWhiteRed {
    White,
    Black,
    Red,
}

impl Color for BlackWhiteRed {
    fn bits_per_pixel() -> usize {
        2
    }
}*/
