use clap::Parser;

fn main() -> Result<(), img2laser::ImageProcessError> {
    let config = img2laser::SinusoidShadingConfig::parse();

    // Set filename for output image, if not in config
    let out_path = match config.output.to_owned() {
        Some(path) => path,
        None => {
            let mut out_path = config.input.clone();
            out_path.set_extension("svg");
            out_path
        }
    };

    // Open image
    let img = image::open(config.input.as_path())?;

    // Process image
    let svg_img = img2laser::process_image(&img, &config);

    // Save image
    svg::save(out_path, &svg_img)?;

    Ok(())
}
