#![allow(non_snake_case)]
use std::io::Cursor;

use base64::{engine::general_purpose, Engine as _};

use dioxus::prelude::*;
use dioxus_web::Config;

use image::{self, DynamicImage, GenericImageView, ImageOutputFormat};
use img2laser::{
    DownloadButton, FileInput, NumberInput, SinusoidShadingConfig, SinusoidSvg, SliderInput,
};

const IMAGE: &[u8] = include_bytes!("../examples/example_1.png");

#[derive(PartialEq, Props)]
struct RootProps {
    img: image::DynamicImage,
}

fn image_to_base64(img: &DynamicImage) -> String {
    let mut image_data: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut image_data), ImageOutputFormat::Png)
        .unwrap();
    let res_base64 = general_purpose::STANDARD.encode(image_data);
    format!("data:image/png;base64,{}", res_base64)
}

fn main() {
    // Open image
    let img = image::load_from_memory(IMAGE).expect("Couldn't load image");

    // launch the web app
    dioxus_web::launch_with_props(App, RootProps { img: img }, Config::new());
}

fn App(cx: Scope<RootProps>) -> Element {
    let (width, height) = cx.props.img.dimensions();

    let config = img2laser::SinusoidShadingConfig {
        width: width as usize,
        height: height as usize,
        ..Default::default()
    };

    use_shared_state_provider(cx, || cx.props.img.clone());
    use_shared_state_provider(cx, || config);

    let img = use_shared_state::<DynamicImage>(cx).unwrap();
    let config = use_shared_state::<SinusoidShadingConfig>(cx).unwrap();

    render! {
        div {
            id: "flex-container",
            div {
                id: "control-panel",
                h1 { "Controls" },
                div {
                    class: "file-input",
                    FileInput {
                        id: "file".to_string(),
                        label: "Select image file: ".to_string(),
                    }
                },
                div {
                    class: "number-input",
                    NumberInput {
                        id: "width".to_string(),
                        label: "SVG width: ".to_string(),
                        min: 1,
                        step: 1,
                    }
                },
                // TODO: Enabling locking height to width (fixed ratio)
                div {
                    class: "number-input",
                    NumberInput {
                        id: "height".to_string(),
                        label: "SVG height: ".to_string(),
                        min: 1,
                        step: 1,
                    }
                },
                div {
                    class: "slider-input",
                    SliderInput {
                        id: "lines".to_string(),
                        label: "Number of lines: ".to_string(),
                        min: 0,
                        max: 128,
                        value: config.read().lines,
                        step: 1,
                        on_input: move |event: FormEvent| {
                            config.with_mut(|c| c.lines = event.value.clone().parse().unwrap_or(64))
                        }
                    }
                },
                div {
                    class: "slider-input",
                    SliderInput {
                        id: "sample_freq".to_string(),
                        label: "Sample frequency: ".to_string(),
                        min: 0.1,
                        max: 10.,
                        value: config.read().sample_freq,
                        step: 0.1,
                        on_input: move |event: FormEvent| {
                            config.with_mut(|c| c.set_field("sample_freq", &event.value.clone()))
                        }
                    }
                },
                div {
                    class: "slider-input",
                    SliderInput {
                        id: "min_freq".to_string(),
                        label: "Minimum frequency: ".to_string(),
                        min: 0.001,
                        max: 0.1,
                        value: config.read().min_freq,
                        step: 0.001,
                        on_input: move |event: FormEvent| {
                            config.with_mut(|c| c.set_field("min_freq", &event.value.clone()))
                        }
                    }
                },
                div {
                    class: "slider-input",
                    SliderInput {
                        id: "max_freq".to_string(),
                        label: "Maximum frequency: ".to_string(),
                        min: 0.1,
                        max: 10.,
                        value: config.read().max_freq,
                        step: 0.1,
                        on_input: move |event: FormEvent| {
                            config.with_mut(|c| {
                                c.set_field("max_freq", &event.value.clone());
                                c.sample_freq = c.max_freq * 4.;
                            })
                        }
                    }
                },
                div {
                    class: "slider-input",
                    SliderInput {
                        id: "amplitude".to_string(),
                        label: "Amplitude: ".to_string(),
                        min: 0.0,
                        max: 0.5,
                        value: config.read().amplitude,
                        step: 0.05,
                        on_input: move |event: FormEvent| {
                            config.with_mut(|c| c.set_field("amplitude", &event.value.clone()))
                        }
                    }
                },
                div {
                    DownloadButton {},
                },
            },
            div {
                id: "output-image-panel",
                h1 { "Output" },
                SinusoidSvg {},
            },
            div {
                id: "input-image-panel",
                h1 { "Input" },
                img { src: "{image_to_base64(&img.read())}" },
            },
        }
    }
}
