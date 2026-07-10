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

    pub fn from_bytes(mut raw: &[u8]) -> Result<Ppm, String> {
        let magic_present = parse_exact(b"P6\n", &mut raw) || parse_exact(b"P6\r\n", &mut raw);
        if !magic_present {
            return Err("PPM magic value missing.".into());
        }

        let pixel_width = parse_number(&mut raw)
            .ok_or("Width missing".to_owned())?;
        let space_present = parse_exact(b" ", &mut raw);
        if !space_present {
            return Err("Space missing between width and height.".into());
        }

        let pixel_height = parse_number(&mut raw)
            .ok_or("Height missing".to_owned())?;
        let correct_max_color_value = parse_exact(b"\n255\n", &mut raw) || parse_exact(b"\r\n255\r\n", &mut raw);
        if !correct_max_color_value {
            return Err("Max color value incorrect or missing.".into());
        }

        let data = raw.to_vec();
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

fn parse_exact(prefix: &[u8], input: &mut &[u8]) -> bool {
    let stripped = input.strip_prefix(prefix);
    if let Some(stripped) = input.strip_prefix(prefix) {
        *input = stripped;
    }

    stripped.is_some()
}

fn parse_number(input: &mut &[u8]) -> Option<usize> {
    let mut output: Option<usize> = None;
    let mut num_len = 0;
    for digit in input.iter() {
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

    *input = &input[num_len..];
    output
}

#[cfg(test)]
mod tests {
    use crate::ppu::pixel_index::PixelIndex;

use super::*;

    #[test]
    fn exact() {
        let mut input = &b"P6\n256"[..];
        assert_eq!(parse_exact(b"P6\n", &mut input), true);
        assert_eq!(input.len(), 3);
    }

    #[test]
    fn exact_failed() {
        let mut input = &b"P3\n256"[..];
        assert_eq!(parse_exact(b"P6\n", &mut input), false);
        assert_eq!(input.len(), 6);
    }

    #[test]
    fn number() {
        let mut input = &b"256abcd"[..];
        assert_eq!(parse_number(&mut input), Some(256));
        assert_eq!(input.len(), 4);
    }

    #[test]
    fn just_number() {
        let mut input = &b"240"[..];
        assert_eq!(parse_number(&mut input), Some(240));
        assert_eq!(input.len(), 0);
    }

    #[test]
    fn bigger_number() {
        let mut input = &b"293abcd"[..];
        assert_eq!(parse_number(&mut input), Some(293));
        assert_eq!(input.len(), 4);
    }

    #[test]
    fn empty_number() {
        let mut input = &b"abcd"[..];
        assert_eq!(parse_number(&mut input), None);
        assert_eq!(input.len(), 4);
    }

    #[test]
    fn number_too_big() {
        let mut input = &b"123123123123123123123123123123123123123123123123abcd"[..];
        assert_eq!(parse_number(&mut input), None);
        assert_eq!(input.len(), 52);
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
