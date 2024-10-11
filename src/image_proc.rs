use std::{error::Error, io};

use ansi_term::Color;
use image::imageops::FilterType;
use image::DynamicImage;
use image::Rgba;

use crate::ascii::DEFAULT;

/// Engine for rendering rgba images to ascii text
///
/// * `source`: DynamicImage
/// * `edge_map`: TODO: implement Edge detection methods
pub struct ImageEngine {
    source: DynamicImage,
    #[allow(unused)]
    edge_map: Option<Vec<(u8, u8)>>,
}

impl ImageEngine {
    /// Construct a new engine from an owned dynamic image
    /// # Usage
    /// ```rust
    ///     
    /// use std::error::Error;
    /// use rustascii::{image, image_proc::ImageEngine};
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let source = image::open("/")?;
    ///     let engine = ImageEngine::new(source);
    ///     // Do your stuff with the engine
    ///     Ok(())
    ///}
    /// ```
    ///
    /// * `source`: constructed `DynamicImage`
    pub fn new(source: DynamicImage) -> Self {
        Self {
            source,
            edge_map: None, // TODO: Implement edge detection
        }
    }

    /// Construct a new engine from a slice of bytes
    /// # Usage
    /// ```rust
    ///     use rustascii::{image_proc::ImageEngine};
    ///     use std::error::Error;
    ///
    ///     fn main() -> Result<(), Box<dyn Error>> {
    ///         let source = include_bytes!("your image path");
    ///         let engine = ImageEngine::from_slice()?;
    ///         // Do stuff with the engine
    ///         Ok(())
    ///     }
    /// ```
    /// * `source`: a slice of bytes
    pub fn from_slice(source: &[u8]) -> Result<Self, Box<dyn Error>> {
        let image = image::load_from_memory(source)?;

        Ok(Self {
            source: image,
            edge_map: None,
        })
    }

    /// Process the image, with scaling, and write the output to a writer.
    ///
    /// Note that either `width` or `height` must be Some(value)
    ///
    /// # Usage
    /// Here is a simple example writing to stdout.
    /// ```rust
    ///     use rustascii::{image_proc::ImageEngine};
    ///     use std::{error::Error, io::stdout};
    ///
    ///     fn main() -> Result<(), Box<dyn Error>> {
    ///         let source = include_bytes!("your-path");
    ///         let engine = ImageEngine::from_slice(source)?;
    ///
    ///         let mut writer = stdout(); // stdout implements io::Write!
    ///
    ///         // If only one of the axis is set,
    ///         // the image aspect ratio will be preserved
    ///         engine.render_to_text(&mut writer, 0, Some(128), None)?;
    ///         Ok(())
    ///     }
    /// ```
    ///
    /// You can also do some more advance with writer, like TcpStream, or File
    /// ```rust
    ///     use rustascii::{image_proc::ImageEngine};
    ///     use std::{error::Error, io::stdout};
    ///
    ///     fn main() -> Result<(), Box<dyn Error>> {
    ///         let source = include_bytes!("your-path");
    ///         let engine = ImageEngine::from_slice(source)?;
    ///
    ///         let mut file_writer = fs::File::create_new("your-new-file")?;
    ///
    ///         // If only one of the axis is set,
    ///         // the image aspect ratio will be preserved
    ///         engine.render_to_text(&mut file_writer, 0, Some(128), None)?;
    ///         Ok(())
    ///     }
    /// ```
    ///
    /// * `writer`: Some thing that implements `io::Write`
    /// * `alpha_threshold`: Lowest possible alpha value for ascii text to be rendered
    /// * `width`: New width of the ascii text
    /// * `height`: New height of the ascii text
    pub fn render_to_text(
        &self,
        writer: &mut dyn io::Write,
        alpha_threshold: u8,
        width: Option<u32>,
        height: Option<u32>,
    ) -> io::Result<()> {
        let (width, height) = self.calculate_dimensions(width, height);
        let image = self
            .source
            .resize_exact(width, height, FilterType::Nearest)
            .to_rgba8();

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

            let char_for_pixel = self.get_char_for_pixel(pixel, alpha_threshold, maximum);
            write!(writer, "{char_for_pixel}")?;
        }

        if let Some(color) = prev_color {
            write!(writer, "{}", color.prefix())?;
        }

        writer.flush()?;

        Ok(())
    }

    /// Get all of the content as a string, using this is not recommended, using `render_to_text`
    /// should covered almost all cases.
    ///
    /// * `alpha_threshold`: Lowest possible alpha value for ascii text to be rendered
    /// * `width`: New width of the ascii text
    /// * `height`: New height of the ascii text
    pub fn get_ascii_as_string(
        &self,
        alpha_threshold: u8,
        width: Option<u32>,
        height: Option<u32>,
    ) -> String {
        let (width, height) = self.calculate_dimensions(width, height);
        let image = self
            .source
            .resize_exact(width, height, FilterType::Nearest)
            .to_rgba8();

        let mut output = String::new();
        let mut prev_color: Option<Color> = None;
        let mut current_line = 0;

        let maximum = image
            .pixels()
            .fold(0.0, |acc, pixel| self.get_grayscale_pixel(pixel).max(acc));

        for (_, line, pixel) in image.enumerate_pixels() {
            if current_line < line {
                current_line = line;
                if let Some(color) = prev_color {
                    output.push_str(&format!("{}", color.suffix()));
                    prev_color = None;
                };
                output.push('\n');
            }

            let color = Color::RGB(pixel[0], pixel[1], pixel[2]);
            if prev_color != Some(color) {
                output.push_str(&format!("{}", color.prefix()));
            }
            prev_color = Some(color);

            let char_for_pixel = self.get_char_for_pixel(pixel, alpha_threshold, maximum);
            output.push_str(&format!("{char_for_pixel}"));
        }

        if let Some(color) = prev_color {
            output.push_str(&format!("{}", color.prefix()));
        }

        output
    }

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
