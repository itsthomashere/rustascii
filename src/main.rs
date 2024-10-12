use std::error::Error;
use std::fs;
use std::io::stdout;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[arg(short, long)]
    width: Option<u32>,

    #[arg(short, long)]
    height: Option<u32>,

    #[arg(short, long)]
    threshold: Option<u8>,

    path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse();

    let width = arguments.width;
    let height = arguments.height;
    if width.is_none() && height.is_none() {
        return Err("either width or height must be set".into());
    }
    let threshold = arguments.threshold.unwrap_or_default();

    let path = PathBuf::from_str(&arguments.path)?;

    let data = fs::read(path)?;

    let image_engine = ascii_rs::image_proc::ImageEngine::from_slice(&data)?;

    let mut writer = stdout();

    image_engine.render_to_text(&mut writer, threshold, width, height)?;

    Ok(())
}
