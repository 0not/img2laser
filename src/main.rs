use std::path::PathBuf;

use clap::Parser;
use image;

/// A program that takes in a bitmap image and outputs a line shaded SVG
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input image path
    input: PathBuf,

    /// Output SVG path
    output: Option<PathBuf>,

    /// Number of lines
    #[arg(short, long, default_value_t = 32)]
    lines: u8,
}

fn process_image<P>(in_path: P, out_path: P) -> image::ImageResult<image::DynamicImage>
where
    P: AsRef<std::path::Path>
{
    let img = image::open(in_path)?;
    let img = img.grayscale();
    img.save(out_path)?;
    return Ok(img.to_owned());
}

fn main() {
    let cli = Cli::parse();

    let mut out_path = cli.input.clone();
    out_path.set_extension("jpg");

    if let Ok(_) = process_image(cli.input.as_path(), out_path.as_path()) {
        println!("Successfully saved image.");
    } else {
        eprintln!("Could not read file at: {}", cli.input.display());
    }

}
