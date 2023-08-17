#![allow(non_snake_case)]
use dioxus::prelude::*;

use image;

const IMAGE: &[u8] = include_bytes!("../examples/example_1.png");

fn main() {
    // launch the web app
    dioxus_web::launch(App);
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let config = img2laser::SinusoidShadingConfig {
        lines: 90,
        min_freq: 0.001,
        max_freq: 2.,
        sample_freq: 5.,
        ..Default::default()
    };

    // Open image
    // let img = image::open(config.input.as_path())?;
    let img = image::load_from_memory(IMAGE).expect("Couldn't load image");

    // Process image
    let svg_img = img2laser::process_image(&img, &config).to_string();

    // Save image
    // svg::save(out_path, &svg_img)?;

    cx.render(rsx! {
        div {
            width: "512px",
            display: "inline-block",
            dangerous_inner_html: "{svg_img}",
        },
    })
}
