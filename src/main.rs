use clap::Parser;

fn main() {
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

    if let Ok(_) = img2laser::process_image(config.input.as_path(), out_path.as_path(), &config) {
        println!("Successfully saved image.");
    } else {
        eprintln!("Could not read file at: {}", config.input.display());
    }
}
