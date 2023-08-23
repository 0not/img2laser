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
// TODO: Add ability to resize image (to speed up subsequent processing)

/// Default values
const LINES: usize = 64;
const WIDTH: usize = 512;
const HEIGHT: usize = 512;
const SAMPLE_FREQ: f32 = 5.;
const MIN_FREQ: f32 = 0.001;
const MAX_FREQ: f32 = 2.;
const AMPLITUDE: f32 = 0.4;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
/// Configuration struct for sine shading process
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

impl SinusoidShadingConfig {
    /// Set a field to a value.
    ///
    /// While writing the Dioxus frontend, I realized I needed a convenient way
    /// to set a field using strings. There may be a better/safer/faster way to
    /// do this, but my case this is sufficient. Using a string slice for the
    /// value enables working with either usize or f32. Since the value comes
    /// from a string (HTML form) in the first place, this seems like an OK
    /// thing to do.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field to modify as a string slice.
    /// * `value` - The new value as a string slice.
    pub fn set_field(&mut self, field: &str, value: &str) {
        match field {
            // usize
            "lines" => self.lines = value.parse().unwrap_or(LINES),
            "width" => self.width = value.parse().unwrap_or(WIDTH),
            "height" => self.height = value.parse().unwrap_or(HEIGHT),
            // f32
            "sample_freq" => self.sample_freq = value.parse().unwrap_or(SAMPLE_FREQ),
            "min_freq" => self.min_freq = value.parse().unwrap_or(MIN_FREQ),
            "max_freq" => self.max_freq = value.parse().unwrap_or(MAX_FREQ),
            "amplitude" => self.amplitude = value.parse().unwrap_or(AMPLITUDE),
            _ => return,
        }
    }

    /// Get a field value as a string.
    ///
    /// See `set_field` for why this function exists.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field to modify as a string slice.
    ///
    /// # Returns
    /// * Field value as a string
    pub fn get_field(&self, field: &str) -> String {
        match field {
            "lines" => self.lines.to_string(),
            "width" => self.width.to_string(),
            "height" => self.height.to_string(),
            "sample_freq" => self.sample_freq.to_string(),
            "min_freq" => self.min_freq.to_string(),
            "max_freq" => self.max_freq.to_string(),
            "amplitude" => self.amplitude.to_string(),
            _ => 0.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
/// Using thiserror was more for the learning experience than necessity.
pub enum ImageProcessError {
    #[error(transparent)]
    ImageError(#[from] image::ImageError),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

/// Convert an image into an SVG using the frequency modulated sinusoidal
/// shading method.
///
/// # Arguments
/// * `img` - A reference to the image. This can be loaded from disk or memory
///   using the `image` crate.
/// * `config` - The configuration struct.
///
/// # Returns
/// * An SVG document (from `svg` crate). This document can be saved to disk or
///   passed to the browser.
pub fn process_image(img: &DynamicImage, config: &SinusoidShadingConfig) -> Document {
    // Spatial sampling frequency
    let fs = config.sample_freq;

    // Output SVG width and height
    let width = config.width;
    let height = config.height;

    // Calculate row height and amplitude. The amplitude in the config struct is
    // a ratio of row height, meaning an amplitude of 0.5 will leave no gap
    // between rows (meaning between sine waves). That is because there is 0.5
    // below the row midpoint and another 0.5 is above it.
    let row_height = height as f32 / config.lines as f32;
    let amp = config.amplitude * row_height;

    // Average over each row and calculate the sinusoid line values.
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

                // This seems to save about 10% to 20% off the SVG size
                // (The SVG as-is has too much wasted precision.)
                //let p = 100.; // Precision
                // vec![(x * p).round() / p, (y * p).round() / p]
                vec![x, y]
            })
            .collect::<Vec<f32>>();

        // Grab the first two values (first x-y pair), since these are needed
        // for the call to `move_to`.
        let x = sine[0];
        let y = sine[1];

        // Add the move to command and the path data to `data`.
        data = data.move_to((x, y));
        data = data.add(Command::Line(Position::Absolute, sine.into()))
    }

    // Create the SVG Step 2:
    //   Create the path using the specified styles and the data from `data`.
    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 1)
        .set("d", data);

    // Create the SVG Step 3:
    //   Finally, create a new document with a viewBox and style. The style is
    //   specified so that the SVG element will scale (down) in the browser.
    let document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("style", format!("width: {}; max-width: 100%;", width))
        .add(path);

    document
}

/// Average the image and get array of size (config.lines, img.width).
///
/// Each sinusoid in the final image is frequency modulated based on the average
/// of the rows that it represents. For an image of height 512 pixels, a lines
/// value of 64 means that 512/64 = 8 rows are used in each average. The average
/// is done vertically (pixel columns) so that the result of the average is a
/// list of length `width`.
///
/// # Arguments
/// * `img` - A reference to the image. This can be loaded from disk or memory
///   using the `image` crate.
/// * `config` - The configuration struct.
///
/// # Returns
/// * A 2D array of size (config.lines, img.width), where each row contains the
///   average for a specific sinusoid.
fn average_rows(img: &DynamicImage, config: &SinusoidShadingConfig) -> Array2<u8> {
    // Grab the image dimensions and force config.lines <= height
    let (width, height) = img.dimensions();
    let lines = if config.lines <= height as usize {
        config.lines
    } else {
        height as usize
    };

    // Calculate the row height and create the zeroed `result` array.
    let row_height = height as f32 / lines as f32;
    let mut result = Array2::zeros((lines, width as usize));

    // Convert img to a grayscale ndarray.
    // Must cast to u32 or 'mean' operation will overflow.
    let img_array = img.to_luma8().into_ndarray2().mapv(|e| u32::from(e));

    // For each line, average `row_height` number of rows and add to `result`
    // ndarray.
    for n in 0..lines {
        // Start at current row (`n`) and end `row_height` later.
        let start = (n as f32 * row_height).round() as usize;
        let end = ((n + 1) as f32 * row_height).round() as usize;

        // Clamp `end` at `height`.
        let end = if end > height as usize {
            height as usize
        } else {
            end
        };

        // Average over the specified rows.
        let row = img_array.slice(s![start..end, ..]).mean_axis(Axis(0));

        // If the mean succeeds, modify `result`. If it fails, keep original
        // zeroes in `result`.
        if let Some(row) = row {
            result.slice_mut(s![n, ..]).assign(&row);
        }
    }

    // This cast shouldn't be too lossy because pre-averaged values were u8
    result.mapv(|e| e as u8)
}

/// Convert averaged image into sine wave array.
///
/// # Arguments
/// * `img` - A reference to the averaged image array.
/// * `config` - The configuration struct.
///
/// # Returns
/// * A 2D array of size (config.lines, img.width * config.sample_freq), where
///   each row contains the frequency modulated sinusoid y-axis values.
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
    let f_min = frequencies.iter().copied().reduce(f32::min).unwrap_or(0.);

    // Global max. frequency from image
    let f_max = frequencies.iter().copied().reduce(f32::max).unwrap_or(0.);

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
        for n in 0..(f.len() - 1) {
            let i = (n as f32 / fs).floor() as usize;
            f[n] = freqs[i];
        }

        // Perform cumulative sum. Add result to phase array (`phi`).
        // For a sine wave, each frequency has a different phase. Therefore,
        // phase must be accumulated to avoid sharp changes when two different
        // frequencies meet.
        // See: https://kylelarsen.com/2021/03/13/sine-wave-line-shading/
        f.accumulate_axis_inplace(Axis(0), |&prev, curr| *curr += prev);
        f /= fs;
        phi.slice_mut(s![r, ..]).assign(&f);
    }

    // Return the sine waves
    phi.mapv(|x| x.sin())
}
