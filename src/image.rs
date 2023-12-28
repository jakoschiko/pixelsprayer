use std::{fs::File, path::Path};

use anyhow::Result;
use fastrand::Rng;
use png::{ColorType, OutputInfo};

use crate::{color::Color, position::Position};

pub struct Image {
    info: OutputInfo,
    frame_size: usize,
    bytes: Vec<u8>,
}

impl Image {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let decoder = png::Decoder::new(file);
        let mut reader = decoder.read_info()?;
        let mut bytes = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut bytes)?;
        let frame_size = info.buffer_size();
        Ok(Self {
            info,
            frame_size,
            bytes,
        })
    }

    pub fn get_random_position(&self, rng: &mut Rng) -> Position {
        let x = rng.u32(0..self.info.width);
        let y = rng.u32(0..self.info.height);
        Position { x, y }
    }

    pub fn get_color(&self, position: Position) -> Option<Color> {
        let pixel_width = self.info.color_type.samples();
        let index =
            (self.info.width as usize * position.y as usize + position.x as usize) * pixel_width;
        let frame = &self.bytes[0..self.frame_size];
        let color = match self.info.color_type {
            ColorType::Grayscale => {
                let c = *frame.get(index)?;
                Color::Grayscale(c)
            }
            ColorType::Rgb => {
                let r = *frame.get(index)?;
                let g = *frame.get(index + 1)?;
                let b = *frame.get(index + 2)?;
                Color::Rgb(r, g, b)
            }
            ColorType::Rgba => {
                let r = *frame.get(index)?;
                let g = *frame.get(index + 1)?;
                let b = *frame.get(index + 2)?;
                let a = *frame.get(index + 3)?;
                Color::Rgba(r, g, b, a)
            }
            ColorType::GrayscaleAlpha => {
                let c = *frame.get(index)?;
                let a = *frame.get(index + 1)?;
                Color::Rgba(c, c, c, a)
            }
            ColorType::Indexed => todo!(),
        };
        Some(color)
    }
}
