use std::{error::Error, io, sync::mpsc::Receiver};

use ansi_term::Color;
use image::{DynamicImage, Rgba};

use crate::ascii::DEFAULT;

pub struct ImageEngine<'a> {
    source: &'a DynamicImage,
    edge_map: Option<Vec<(u8, u8)>>,
}

impl<'a> ImageEngine<'a> {
    pub fn new(source: &'a DynamicImage) -> Self {
        Self {
            source,
            edge_map: None, // TODO: Implement edge detection
        }
    }

    pub fn render_to_text(
        &self,
        writer: &mut dyn io::Write,
        width: Option<u32>,
        height: Option<u32>,
    ) -> io::Result<()> {
        let (width, height) = self.calculate_dimensions(width, height);
        let image = self.source.thumbnail_exact(width, height).to_rgba8();

        let mut prev_color: Option<Color> = None;
        let mut current_line = 0;

        let maximum = image
            .pixels()
            .fold(0.0, |acc, pixel| self.get_grayscale_pixel(pixel).max(acc));

        for (_, line, pixel) in image.enumerate_pixels() {
            if current_line < line {
                current_line = line;
                if let Some(color) = prev_color {
                    write!(writer, "{}", color.suffix())?;
                    prev_color = None;
                };
                writeln!(writer)?;
            }

            let color = Color::RGB(pixel[0], pixel[1], pixel[2]);
            if prev_color != Some(color) {
                write!(writer, "{}", color.prefix())?;
            }
            prev_color = Some(color);

            let char_for_pixel = self.get_char_for_pixel(pixel, 0, maximum);
            write!(writer, "{char_for_pixel}")?;
        }

        if let Some(color) = prev_color {
            write!(writer, "{}", color.prefix())?;
        }

        writer.flush()?;

        Ok(())
    }

    // pub fn render_to_image(
    //     &self,
    //     path: String,
    //     width: Option<u32>,
    //     height: Option<u32>,
    // ) -> Result<(), Box<dyn Error>> {
    //     let (width, height) = self.calculate_dimensions(width, height);
    //     let mut new_image = image::RgbaImage::new(width, height);
    //
    //     new_image.put_pixel(x, y, pixel);
    //     Ok(())
    // }

    // fn channel_process(&self, width: u32, height: u32) -> Receiver<_> {}

    fn get_char_for_pixel(&self, pixel: &Rgba<u8>, alpha_threshold: u8, maximum: f64) -> char {
        let gray_scale = self.get_grayscale_pixel(pixel) / maximum;
        if pixel.0[3] <= alpha_threshold {
            return ' ';
        }

        DEFAULT[(gray_scale * (DEFAULT.len() - 1) as f64) as usize]
    }

    fn get_grayscale_pixel(&self, pixel: &Rgba<u8>) -> f64 {
        ((pixel.0[0] as f64) * 0.2989)
            + (pixel.0[1] as f64 * 0.5870)
            + ((pixel.0[2] as f64) * 0.1140) / 255.0
    }

    fn calculate_dimensions(&self, width: Option<u32>, height: Option<u32>) -> (u32, u32) {
        (
            width.unwrap_or_else(|| {
                (height.expect("Either width or weight must be specified") as f64
                    * self.source.width() as f64
                    / self.source.height() as f64
                    / 2.0)
                    .ceil() as u32
            }),
            height.unwrap_or_else(|| {
                (width.expect("Either height or width must be specified") as f64
                    * self.source.height() as f64
                    / self.source.width() as f64
                    / 2.0)
                    .ceil() as u32
            }),
        )
    }
}
