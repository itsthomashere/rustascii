use std::env::args;
use std::error::Error;
use std::str::FromStr;
use std::{
    fs,
    io::{self},
    path::PathBuf,
};

use rustascii::image_proc::ImageEngine;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_arguments()?;

    let mut writer = io::stdout();

    let data = fs::read(args.path)?;

    let engine = ImageEngine::from_slice(&data)?;
    engine.render_to_text(&mut writer, args.width, args.height)?;

    Ok(())
}

struct Arguments {
    path: PathBuf,
    width: Option<u32>,
    height: Option<u32>,
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut height: Option<u32> = None;
    let mut width: Option<u32> = None;
    let mut path: Option<PathBuf> = None;
    for arg in args().skip(1).take(3) {
        if arg.starts_with("--width=") {
            width = Some(parse_width(arg)?);
            continue;
        } else if arg.starts_with("--height=") {
            height = Some(parse_height(arg)?);
            continue;
        } else if arg.starts_with("/") {
            path = Some(PathBuf::from_str(&arg).map_err(|_| "Invalid path".to_string())?);
        } else {
            return Err("Invalid arguments".to_string());
        }
    }

    Ok(Arguments {
        path: path.expect("Invalid path"),
        width,
        height,
    })
}

fn parse_width(arg: String) -> Result<u32, String> {
    arg.strip_prefix("--width=")
        .unwrap()
        .trim()
        .parse::<u32>()
        .map_err(|_| "invalid width".to_string())
}

fn parse_height(arg: String) -> Result<u32, String> {
    arg.strip_prefix("--height=")
        .unwrap()
        .trim()
        .parse::<u32>()
        .map_err(|_| "invalid width".to_string())
}
