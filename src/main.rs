use std::path::PathBuf;

use clap::Parser;
use image::{self, GenericImage, DynamicImage, imageops, GenericImageView, ImageBuffer};

/// Default values
const LINES: u32 = 32;

/// A program that takes in a bitmap image and outputs a line shaded SVG
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input image path
    input: PathBuf,

    /// Output SVG path
    output: Option<PathBuf>,

    /// Number of lines
    #[arg(short, long, default_value_t = LINES)]
    lines: u32,
}

struct LineShadingConfig {
    lines: u32,
}

impl Default for LineShadingConfig {
    fn default() -> Self {
        LineShadingConfig { lines: LINES }
    }
}

fn process_image<P>(
    in_path: P, 
    out_path: P, 
    config: &LineShadingConfig,
) -> image::ImageResult<image::DynamicImage>
where
    P: AsRef<std::path::Path>
{
    // Open image
    let img = image::open(in_path)?;

    // Convert to grayscale. Using `grayscale()` was leaving the image in Rgba8,
    // which means we'd have to deal with 3 channels.
    let img = img.into_luma8();

    // Grab the image dimensions and create an empty image buffer to store the
    // averaged rows. 
    let (width, height) = img.dimensions();
    let mut out = ImageBuffer::new(width, height);

    // Vertical height of each row
    let row_h = height / config.lines;
    
    // Iterate over the rows
    for y in (0..height).step_by(row_h as usize) {
        // Iterate over the columns in each row
        for x in 0..width {
            // Grab a 1 pixel wide vertical slice of each row
            let row = img.view(x, y, 1, row_h);
            let (w, h) = row.dimensions();

            // Average the vertical slice
            let avg = row
                .pixels()
                .map(|(_, _, pixel)| {
                    pixel.0[0] as u32
                })
                .sum::<u32>() / (w * h);
            let avg: u8 = avg.try_into().unwrap();

            // Write the average value to the pixels that were averaged
            for yd in 0..row_h {
                out.put_pixel(x, y + yd, image::Luma([avg]));
            }
        }
    }

    out.save(out_path)?;

    return Ok(DynamicImage::ImageLuma8(out));
}

fn average_rows(img: DynamicImage) -> DynamicImage {
    return img;
}

fn main() {
    let cli = Cli::parse();

    let mut out_path = cli.input.clone();
    out_path.set_extension("jpg");

    let config = LineShadingConfig {
        lines: cli.lines,
    };

    if let Ok(_) = process_image(
        cli.input.as_path(), 
        out_path.as_path(),
        &config,
    ) {
        println!("Successfully saved image.");
    } else {
        eprintln!("Could not read file at: {}", cli.input.display());
    }

}
