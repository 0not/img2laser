use std::path::PathBuf;

use clap::Parser;

use image::{DynamicImage, GenericImageView};

use ndarray::{s, Array1, Array2, Axis};

use nshare::ToNdarray2;

use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

// TODO: Add support for transparency (locations where no line will be drawn)
// TODO: Add support for vertical sinusoids.

/// Default values
const LINES: usize = 64;
const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const SAMPLE_FREQ: f32 = 10.;
const MIN_FREQ: f32 = 0.001;
const MAX_FREQ: f32 = 0.5;
const AMPLITUDE: f32 = 0.4;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct SinusoidShadingConfig {
    /// Input image path
    pub input: PathBuf,

    /// Output SVG path
    pub output: Option<PathBuf>,

    /// Number of sinusoids, or rows, to create
    #[arg(long, default_value_t = LINES)]
    pub lines: usize,

    /// Output image width
    #[arg(long, default_value_t = WIDTH)]
    pub width: usize,

    /// Output image height  
    #[arg(long, default_value_t = HEIGHT)]
    pub height: usize,

    /// Spatial sample frequency. A larger number means the resulting sinusoid
    /// will contain more points.
    #[arg(long, default_value_t = SAMPLE_FREQ)]
    pub sample_freq: f32,

    /// Minimum sinusoid frequency
    #[arg(long, default_value_t = MIN_FREQ)]
    pub min_freq: f32,

    /// Maximum sinusoid frequency
    #[arg(long, default_value_t = MAX_FREQ)]
    pub max_freq: f32,

    /// Sinusoid amplitude (when constant). Should be less than 0.5 to avoid
    /// overlapping sinusoids.
    #[arg(long, default_value_t = AMPLITUDE)]
    pub amplitude: f32,
}

impl Default for SinusoidShadingConfig {
    fn default() -> Self {
        SinusoidShadingConfig {
            input: PathBuf::from("image.png"),
            output: Some(PathBuf::from("image.svg")),
            lines: LINES,
            width: WIDTH,
            height: HEIGHT,
            sample_freq: SAMPLE_FREQ,
            min_freq: MIN_FREQ,
            max_freq: MAX_FREQ,
            amplitude: AMPLITUDE,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImageProcessError {
    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn process_image(img: &DynamicImage, config: &SinusoidShadingConfig) -> Document {
    // Spatial sampling frequency
    let fs = config.sample_freq;
    let width = config.width;
    let height = config.height;
    let row_height = height as f32 / config.lines as f32;
    let amp = config.amplitude * row_height;

    // Average the rows
    let avgs = average_rows(&img, config);
    let lines = make_lines(&avgs, config);

    // Create the SVG Step 1:
    //   Create the data for the path. The SVG path data consists of a list of
    //   x/y coordinates in the format: x0, y0, x1, y1, x2, y2 ...
    let mut data = Data::new();
    for (yi, row) in lines.axis_iter(Axis(0)).enumerate() {
        // `y_offset` increases for each row by `row_height`. A global shift of
        // 0.5 is added to `yi` so that the first sinusoid doesn't overflow the
        // top boundary.
        let y_offset = (0.5 + yi as f32) * row_height;

        // x_max is used to properly scale the x-values so the width is the
        // value provided by the user
        let x_max = row.len() as f32 / fs;

        // `sine` is the data passed to the SVG path
        let sine = row
            .iter()
            .enumerate()
            .flat_map(|(xi, &y)| {
                let x_scale = width as f32 / x_max;

                let x = x_scale * (xi as f32 / fs);
                let y = amp * y + y_offset;

                vec![x, y]
            })
            .collect::<Vec<f32>>();

        let x = sine[0];
        let y = sine[1];

        data = data.move_to((x, y));
        data = data.add(Command::Line(Position::Absolute, sine.into()))
    }

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data);

    let document = Document::new().add(path);

    // svg::save(out_path, &document)?;

    // Ok(())
    document
}

fn average_rows(img: &DynamicImage, config: &SinusoidShadingConfig) -> Array2<u8> {
    // Grab the image dimensions and force config.lines <= height
    let (width, height) = img.dimensions();

    let lines = if config.lines <= height as usize {
        config.lines
    } else {
        height as usize
    };

    let row_height = height as usize / lines;
    let mut result = Array2::zeros((lines, width as usize));

    // Must cast to u32 or 'mean' operation will overflow.
    let img_array = img.to_luma8().into_ndarray2().mapv(|e| u32::from(e));

    for n in 0..lines {
        let start = n * row_height;
        let end = (n + 1) * row_height;
        let row: Array1<u32> = img_array
            .slice(s![start..end, ..])
            .mean_axis(Axis(0))
            .unwrap(); // TODO: Handle safely?

        result.slice_mut(s![n, ..]).assign(&row);
    }

    // This cast shouldn't be lossy because pre-averaged values were u8
    result.mapv(|e| e as u8)
}

fn make_lines(img: &Array2<u8>, config: &SinusoidShadingConfig) -> Array2<f32> {
    // Spatial "sampling frequency". If lower, the processing
    //  will be faster, but at the sake of poorer spatial resolution
    //  (sine waves won't look like sine waves)
    let fs: f32 = config.sample_freq;

    // The spatial frequency will be scaled to be within these bounds
    let f_min_new: f32 = config.min_freq;
    let f_max_new: f32 = config.max_freq;

    let (rows, cols) = img.dim();

    // Horizontal sample locations
    let x = Array1::range(0., cols as f32, 1. / fs);

    // The phase of the sine waves
    let mut phi = Array2::zeros((rows, x.len()));

    // The frequencies come from the image pixel values (intensity)
    // let frequencies = (u8::MAX - img) / u8::MAX;
    let frequencies = img.mapv(|x| f32::from(u8::MAX - x) / f32::from(u8::MAX));

    // Global min. frequency from image
    let f_min = frequencies.iter().copied().reduce(f32::min).unwrap();

    // Global max. frequency from image
    let f_max = frequencies.iter().copied().reduce(f32::max).unwrap();

    for r in 0..rows {
        // Linearly scale the frequencies in to the new range.
        let f_slice = frequencies.slice(s![r, ..]);
        let scale = if f_max - f_min != 0. {
            (f_max_new - f_min_new) / (f_max - f_min)
        } else {
            1.
        };
        let freqs: Array1<f32> = f_slice.mapv(|f| f_min_new + scale * (f - f_min));

        // Initialize the frequency array to zeros
        let mut f = Array1::<f32>::zeros(x.len());

        // Loop through the f array and sample the value from the freqs array
        for n in 0..f.len() {
            let i = (n as f32 / fs).floor() as usize;
            f[n] = freqs[i];
        }

        f.accumulate_axis_inplace(Axis(0), |&prev, curr| *curr += prev);
        f /= fs;
        phi.slice_mut(s![r, ..]).assign(&f);
    }

    // Return the sine waves
    phi.mapv(|x| x.sin())
}
