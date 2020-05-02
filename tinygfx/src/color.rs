pub trait Color: Copy + Clone {
    fn bits_per_pixel() -> usize;
    fn fill(&self, buffer: &mut [u8], left: i32, right: i32);
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

    fn mirror_x(buffer: &mut [u8], width: usize) {
        let bytes = (width + 7) >> 3;
        for i in 0..bytes / 2 {
            let temp = buffer[i].reverse_bits();
            buffer[i] = buffer[bytes - i - 1].reverse_bits();
            buffer[bytes - i - 1] = temp;
        }
        if (bytes & 1) != 0 {
            let center = bytes / 2 + 1;
            buffer[center] = buffer[center].reverse_bits();
        }
        // If the first byte is now incomplete, shift to the left.
        let remainder = width & 0x7;
        if remainder != 0 {
            let shift = 8 - remainder;
            for i in 0..(bytes - 1) {
                buffer[i] = (buffer[i] << shift) | (buffer[i + 1] >> remainder);
            }
            buffer[bytes - 1] <<= shift;
        }
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
