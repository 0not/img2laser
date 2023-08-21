#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_web::Config;

use image::{self, EncodableLayout, GenericImageView};

const IMAGE: &[u8] = include_bytes!("../examples/example_1.png");
const MAX_DIM: u32 = 1024;

#[derive(PartialEq, Props)]
struct RootProps {
    img: image::DynamicImage,
}

fn main() {
    // Open image
    let img = image::load_from_memory(IMAGE).expect("Couldn't load image");

    // launch the web app
    dioxus_web::launch_with_props(App, RootProps { img: img }, Config::new());
}

#[inline_props]
fn SliderInput<'a, T: std::fmt::Display>(
    cx: Scope<'a>,
    id: String,
    label: String,
    min: T,
    max: T,
    // value: &'a str,
    value: T,
    step: T,
    on_input: EventHandler<'a, FormEvent>,
) -> Element {
    let len_min = format!("{}", min).len();
    let len_max = format!("{}", max).len();
    let width = if len_min > len_max { len_min } else { len_max };

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
            step: "{step}",
            value: "{value}",
            oninput: move |event| on_input.call(event),
        },
        input {
            r#type: "number",
            id: "{id}-exact",
            min: "{min}",
            step: "{step}",
            value: "{value}",
            style: "width: {width}em",
            oninput: move |event| on_input.call(event),
        }
    }
}

#[inline_props]
fn SinusoidSvg<'a>(
    cx: Scope<'a>,
    img: &'a image::DynamicImage,
    config: &'a img2laser::SinusoidShadingConfig,
) -> Element {
    // Process image
    let svg_img = img2laser::process_image(&img, &config);

    render! {
            div {
                width: "{config.width}px",
                max_width: "100%",
                display: "inline-block",
                dangerous_inner_html: "{svg_img.to_string()}",
        }
    }
}

fn App(cx: Scope<RootProps>) -> Element {
    let file: &UseState<Vec<u8>> = use_state(cx, Vec::new);
    let img = use_state(cx, || cx.props.img.clone());

    let (width, height) = img.dimensions();

    let config = use_ref(cx, || img2laser::SinusoidShadingConfig {
        lines: 64,
        width: width as usize,
        height: height as usize,
        ..Default::default()
    });

    render! {
        div {
            label {
                r#for: "file",
                "Select image file: "
            },
            input {
                r#type: "file",
                id: "file",
                multiple: false,
                directory: false,
                accept: ".png,.jpg,.jpeg,.gif,.tif,.tiff",
                // See example: https://github.com/DioxusLabs/dioxus/blob/master/examples/file_upload.rs
                onchange: |event| {
                    to_owned![file, img, config];
                    async move {
                        if let Some(file_engine) = &event.files {
                            let files = file_engine.files();

                            let file_path = match files.get(0) {
                                Some(file) => file,
                                None => return,
                            };

                            let file_contents: Vec<u8> = match file_engine.read_file(file_path).await {
                                Some(contents) => contents,
                                None => return,
                            };

                            let mut tmp_img = image::load_from_memory(file_contents.as_bytes())
                                .expect("Could not load image");

                            // Resize image if too large
                            let (w, h) = tmp_img.dimensions();

                            if w > MAX_DIM || h > MAX_DIM {
                                tmp_img = tmp_img.resize(MAX_DIM, MAX_DIM, image::imageops::FilterType::Triangle);
                            }

                            let (width, height) = tmp_img.dimensions();

                            img.set(tmp_img);
                            file.set(file_contents);

                            // Update dimensions
                            config.with_mut(|c| {
                                c.width = width as usize;
                                c.height = height as usize;
                            });
                        }
                    }
                }
            }
        },
        div {
            label {
                r#for: "width",
                "SVG width: "
            },
            input {
                r#type: "number",
                id: "width",
                min: "1",
                step: "1",
                value: "{config.read().width}",
                style: "width: 5em",
                onchange: move |event: FormEvent| {
                    config.with_mut(|c| c.width = event.value.clone().parse().unwrap_or(512))
                },
            },
        },
        // TODO: Enabling locking height to width (fixed ratio)
        div {
            label {
                r#for: "height",
                "SVG height: "
            },
            input {
                r#type: "number",
                id: "height",
                min: "1",
                step: "1",
                value: "{config.read().height}",
                style: "width: 5em",
                onchange: move |event: FormEvent| {
                    config.with_mut(|c| c.height = event.value.clone().parse().unwrap_or(512))
                },
            },
        },
        div {
            SliderInput {
                id: String::from("lines"),
                label: String::from("Number of lines: "),
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
            SliderInput {
                id: String::from("sample-freq"),
                label: String::from("Sample frequency: "),
                min: 0.1,
                max: 10.,
                value: config.read().sample_freq,
                step: 0.1,
                on_input: move |event: FormEvent| {
                    config.with_mut(|c| c.sample_freq = event.value.clone().parse().unwrap_or(0.001))
                }
            }
        },
        div {
            SliderInput {
                id: String::from("min-freq"),
                label: String::from("Minimum frequency: "),
                min: 0.001,
                max: 0.1,
                value: config.read().min_freq,
                step: 0.001,
                on_input: move |event: FormEvent| {
                    config.with_mut(|c| c.min_freq = event.value.clone().parse().unwrap_or(0.001))
                }
            }
        },
        div {
            SliderInput {
                id: String::from("max-freq"),
                label: String::from("Maximum frequency: "),
                min: 0.1,
                max: 10.,
                value: config.read().max_freq,
                step: 0.1,
                on_input: move |event: FormEvent| {
                    config.with_mut(|c| {
                        c.max_freq = event.value.clone().parse().unwrap_or(2.);
                        c.sample_freq = c.max_freq * 4.;
                    })
                }
            }
        },
        div {
            SliderInput {
                id: String::from("amplitude"),
                label: String::from("Amplitude: "),
                min: 0.0,
                max: 0.5,
                value: config.read().amplitude,
                step: 0.05,
                on_input: move |event: FormEvent| {
                    config.with_mut(|c| c.amplitude = event.value.clone().parse().unwrap_or(2.))
                }
            }
        },
        SinusoidSvg {
            config: &*config.read(),
            img: &img,
        }
    }
}
