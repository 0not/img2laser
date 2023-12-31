# img2laser: Sine wave line shading
`img2laser` is a [browser-based tool](http://0not.net/img2laser/) written entirely in Rust that can convert a bitmap image into a laser-ready SVG image using the sine wave line shading technique. In my [original blog post](https://kylelarsen.com/2021/03/13/sine-wave-line-shading/), I showcased a [Python notebook](https://github.com/0not/laser_tools/blob/main/line_shading.ipynb) that performed the same task, but it was lacking in portability. I wanted to make the tool more useful for others, and `img2laser` is the result. Rust is a great language for deploying 100% client-side web apps because of the compiler's amazing support for WebAssembly. Once you load img2laser you could disable your internet connection and it would still work.

# Build instructions
## Build for CLI
To build the CLI tool, simply run `cargo build --bin img2laser-cli`. The tool has a help message for instructions.

## Build for web
To run locally, install dioxus-cli with `cargo install dioxus-cli` and run `dx serve --release`. I recommend building with `--release` to increase the responsiveness of the web interface. A release build for uploading elsewhere can be built with `dx build --release`.


# Example
![baboon](examples/example_1.png "Original baboon image")
![sine wave baboon](examples/example_1.svg "Sine wave baboon image")