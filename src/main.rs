#![allow(non_snake_case)]
use dioxus::prelude::*;

use image;

const IMAGE: &[u8] = include_bytes!("../examples/example_1.png");

fn main() {
    // launch the web app
    dioxus_web::launch(App);
}

#[inline_props]
fn SliderInput<'a>(
    cx: Scope<'a>,
    id: String,
    label: String,
    min: usize,
    max: usize,
    value: &'a str,
    step: usize,
    on_input: EventHandler<'a, FormEvent>,
) -> Element {
    render! {
        label {
            r#for: "{id}",
            "{label}"
        },
        input {
            r#type: "range",
            id: "{id}",
            min: "{min}",
            max: "{max}",
            value: "{value}",
            step: "{step}",
            oninput: move |event| on_input.call(event),
        },
        input {
            r#type: "number",
            id: "{id}+-exact",
            value: "{value}",
            style: "width: 3em",
            oninput: move |event| on_input.call(event),
        }
    }
}

#[inline_props]
fn SinusoidSvg<'a>(cx: Scope<'a>, lines: &'a str) -> Element {
    let lines: usize = lines.parse().unwrap_or(64);

    let config = img2laser::SinusoidShadingConfig {
        lines: lines,
        min_freq: 0.001,
        max_freq: 2.,
        sample_freq: 5.,
        ..Default::default()
    };

    // Open image
    let img = image::load_from_memory(IMAGE).expect("Couldn't load image");

    // Process image
    let svg_img = img2laser::process_image(&img, &config);

    render! {
            div {
                width: "{config.width}px",
                display: "inline-block",
                dangerous_inner_html: "{svg_img.to_string()}",
        }
    }
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let lines = use_state(cx, || "64".to_string());

    render! {
        div {
            SliderInput {
                id: String::from("lines"),
                label: String::from("Number of lines: "),
                min: 0,
                max: 128,
                value: lines,
                step: 1,
                on_input: move |event: FormEvent| {lines.set(event.value.clone())}
            }
        },
        div {
            button {
                id: "generate-svg",
                "Generate"
            }
        },
        SinusoidSvg {
            lines: lines,
        }
    }
}
