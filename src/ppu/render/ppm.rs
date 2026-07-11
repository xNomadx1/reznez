use std::hash::Hash;
// Portable PixMap binary (P6 not P3) file format.
#[derive(PartialEq, Eq, Hash)]
pub struct Ppm {
    data: Vec<u8>,
    pixel_width: usize,
    pixel_height: usize,
}

impl Ppm {
    pub fn new(data: Vec<u8>, pixel_width: usize, pixel_height: usize) -> Ppm {
        assert_eq!(data.len(), 3 * pixel_width * pixel_height);
        Self { data, pixel_width, pixel_height }
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Ppm, String> {
        let mut parser = Parser::new(raw);
        let magic_present = parser.exact(b"P6\n") || parser.exact(b"P6\r\n");
        if !magic_present {
            return Err("PPM magic value missing.".into());
        }

        let pixel_width = parser.number()
            .ok_or("Width missing".to_owned())?;
        let space_present = parser.exact(b" ");
        if !space_present {
            return Err("Space missing between width and height.".into());
        }

        let pixel_height = parser.number()
            .ok_or("Height missing".to_owned())?;
        let correct_max_color_value = parser.exact(b"\n255\n") || parser.exact(b"\r\n255\r\n");
        if !correct_max_color_value {
            return Err("Max color value incorrect or missing.".into());
        }

        let data = parser.remainder().to_vec();
        let expected_len = 3 * pixel_width * pixel_height;
        if data.len() != expected_len {
            return Err(format!("Expected PPM data length to be {} but was {}.", expected_len, data.len()));
        }

        Ok(Self { data, pixel_width, pixel_height })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let metadata = &format!("P6\n{} {}\n255\n", self.pixel_width, self.pixel_height).into_bytes();
        let mut bytes = Vec::with_capacity(metadata.len() + self.data_size());
        bytes.extend_from_slice(metadata);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    fn pixel_count(&self) -> usize {
        self.pixel_width * self.pixel_height
    }

    fn data_size(&self) -> usize {
        3 * self.pixel_count()
    }
}

struct Parser<'a> {
    data: &'a [u8],
}

impl <'a> Parser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn exact(&mut self, prefix: &[u8]) -> bool {
        let stripped = self.data.strip_prefix(prefix);
        if let Some(stripped) = self.data.strip_prefix(prefix) {
            self.data = stripped;
        }

        stripped.is_some()
    }

    fn number(&mut self) -> Option<usize> {
        let mut output: Option<usize> = None;
        let mut num_len = 0;
        for digit in self.data.iter() {
            if digit.is_ascii_digit() {
                let digit = *digit - b'0';
                if let Some(output) = &mut output {
                    *output = output.checked_mul(10)?;
                    *output = output.checked_add(digit as usize)?;
                } else {
                    output = Some(digit as usize);
                }
            } else {
                break;
            }

            num_len += 1;
        }

        self.data = &self.data[num_len..];
        output
    }

    fn remainder(&self) -> &[u8] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use crate::ppu::pixel_index::PixelIndex;

use super::*;

    #[test]
    fn exact() {
        let input = &b"P6\n256"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.exact(b"P6\n"), true);
        assert_eq!(parser.remainder().len(), 3);
    }

    #[test]
    fn exact_failed() {
        let input = &b"P3\n256"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.exact(b"P6\n"), false);
        assert_eq!(parser.remainder().len(), 6);
    }

    #[test]
    fn number() {
        let input = &b"256abcd"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.number(), Some(256));
        assert_eq!(parser.remainder().len(), 4);
    }

    #[test]
    fn just_number() {
        let input = &b"240"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.number(), Some(240));
        assert_eq!(parser.remainder().len(), 0);
    }

    #[test]
    fn bigger_number() {
        let input = &b"293abcd"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.number(), Some(293));
        assert_eq!(parser.remainder().len(), 4);
    }

    #[test]
    fn empty_number() {
        let input = &b"abcd"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.number(), None);
        assert_eq!(parser.remainder().len(), 4);
    }

    #[test]
    fn number_too_big() {
        let input = &b"123123123123123123123123123123123123123123123123abcd"[..];
        let mut parser = Parser::new(input);
        assert_eq!(parser.number(), None);
        assert_eq!(parser.remainder().len(), 52);
    }

    #[test]
    fn roundtrip() {
        let mut data = Vec::with_capacity(3 * PixelIndex::PIXEL_COUNT);
        for i in 0..3 * PixelIndex::PIXEL_COUNT {
            data.push((i % 256) as u8);
        }

        let ppm = Ppm::new(data.clone(), 256, 240);
        let bytes = &ppm.to_bytes();
        assert_eq!(&bytes[bytes.len() - data.len()..], &data);
        let ppm = Ppm::from_bytes(bytes).unwrap();
        let bytes = &ppm.to_bytes();
        assert_eq!(&bytes[bytes.len() - data.len()..], &data);
    }
}
