use core::panic;

use image::{Pixel, Rgb, Rgba};

pub struct Ascii {
    color: Rgb<u8>,
    ch: AsciiChar,
}

#[repr(transparent)]
pub struct AsciiChar(pub char);

impl From<u8> for AsciiChar {
    fn from(value: u8) -> Self {
        match value {
            0..10 => AsciiChar(' '),
            _ => panic!(),
        }
    }
}

impl From<&Rgba<u8>> for Ascii {
    fn from(value: &Rgba<u8>) -> Self {
        Self {
            color: value.to_rgb(),
            ch: value.0[3].into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lumi_to_ascii() {
        let ascii: AsciiChar = 8.into();
        assert_eq!(ascii.0, ' ')
    }
}
