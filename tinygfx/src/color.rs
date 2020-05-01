pub trait Color: Copy + Clone {
    fn bits_per_pixel() -> usize;
    fn fill(&self, buffer: &mut [u8], left: i32, right: i32);
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
