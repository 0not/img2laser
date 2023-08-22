#![allow(non_snake_case)]
use crate::SinusoidShadingConfig;
use dioxus::prelude::*;
use image::{DynamicImage, EncodableLayout, GenericImageView};

const MAX_DIM: u32 = 1024;

#[inline_props]
pub fn SinusoidSvg(cx: Scope) -> Element {
    let img = use_shared_state::<DynamicImage>(cx).unwrap();
    let config = use_shared_state::<SinusoidShadingConfig>(cx).unwrap();

    // Process image
    let svg_img = crate::process_image(&img.read(), &config.read());

    render! {
            div {
                id: "svg-container",
                dangerous_inner_html: "{svg_img.to_string()}",
        }
    }
}

#[inline_props]
pub fn SliderInput<'a, T: std::fmt::Display>(
    cx: Scope<'a>,
    id: String,
    label: String,
    min: T,
    max: T,
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
pub fn FileInput(cx: Scope, id: String, label: String) -> Element {
    let img = use_shared_state::<DynamicImage>(cx).unwrap();
    let config = use_shared_state::<SinusoidShadingConfig>(cx).unwrap();

    render! {
        label {
            r#for: "{id}",
            "{label}"
        },
        input {
            r#type: "file",
            id: "{id}",
            multiple: false,
            directory: false,
            accept: ".png,.jpg,.jpeg,.gif,.tif,.tiff",
            // See example: https://github.com/DioxusLabs/dioxus/blob/master/examples/file_upload.rs
            onchange: |event| {
                to_owned![img, config];
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

                        img.with_mut(|i| *i = tmp_img);
                        // file.set(file_contents);

                        // Update dimensions
                        config.with_mut(|c| {
                            c.width = width as usize;
                            c.height = height as usize;
                        });
                    }
                }
            }
        }
    }
}

#[inline_props]
pub fn NumberInput<T: std::fmt::Display>(
    cx: Scope,
    id: String,
    label: String,
    min: T,
    step: T,
) -> Element {
    let config = use_shared_state::<SinusoidShadingConfig>(cx).unwrap();
    let value = config.read().get_field(id);

    render! {
        label {
            r#for: "{id}",
            "{label}"
        },
        input {
            r#type: "number",
            id: "{id}",
            min: "{min}",
            step: "{step}",
            value: "{value}",
            style: "width: 5em",
            onchange: move |event| config.with_mut(|c| c.set_field(&id, &event.value.clone())),
        },
    }
}

pub fn DownloadButton(cx: Scope) -> Element {
    let create_eval = use_eval(cx);

    render! {
        input {
            r#type: "button",
            id: "download",
            value: "Download SVG",
            onclick: move |_| {
                create_eval(
                    r#"
                    // https://stackoverflow.com/a/46403589
                    function saveSvg(svgEl, name) {
                        svgEl.setAttribute("xmlns", "http://www.w3.org/2000/svg");
                        var svgData = svgEl.outerHTML;
                        var preface = '<?xml version="1.0" standalone="no"?>\r\n';
                        var svgBlob = new Blob([preface, svgData], {type:"image/svg+xml;charset=utf-8"});
                        var svgUrl = URL.createObjectURL(svgBlob);
                        var downloadLink = document.createElement("a");
                        downloadLink.href = svgUrl;
                        downloadLink.download = name;
                        document.body.appendChild(downloadLink);
                        downloadLink.click();
                        document.body.removeChild(downloadLink);
                    }
                    saveSvg(document.getElementById('svg-container').children[0], 'sine_shaded_image.svg');
                    "#
                ).unwrap();
            }
        }
    }
}
