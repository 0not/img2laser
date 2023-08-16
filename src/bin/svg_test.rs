use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

use std::io;

struct SvgConfig {
    width: usize,
    height: usize,
    fs: f32, // spatial sample frequency
    start_x: f32,
    start_y: f32,
    amp: f32,
}

impl Default for SvgConfig {
    fn default() -> Self {
        SvgConfig {
            width: 100,
            height: 100,
            fs: 10., // spatial sample frequency
            start_x: 0.,
            start_y: 2.,
            amp: 1.8,
        }
    }
}

fn save_svg(c: &SvgConfig) -> io::Result<()> {
    // Create sine wave
    let sine1 = (0..c.width * (c.fs as usize))
        .flat_map(|xi| {
            let x = c.start_x + (xi as f32) / c.fs;
            let y = c.start_y + c.amp * (x).sin();
            vec![x, y]
        })
        .collect::<Vec<f32>>();

    let sine2 = (0..c.width * (c.fs as usize))
        .flat_map(|xi| {
            let x = c.start_x + (xi as f32) / c.fs;
            let y = 6. + c.amp * x.sin();
            vec![x, y]
        })
        .collect::<Vec<f32>>();

    // let data = Data::new()
    //     .move_to((0, 2))
    //     .add(Command::Line(Position::Absolute, sine1.into()))
    //     .move_to((0, 6))
    //     .add(Command::Line(Position::Absolute, sine2.into()));

    let mut data = Data::new();
    data = data.move_to((0, 2));
    data = data.add(Command::Line(Position::Absolute, sine1.into()));
    data = data.move_to((0, 6));
    data = data.add(Command::Line(Position::Absolute, sine2.into()));

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 0.1)
        .set("d", data);

    let document = Document::new()
        .set("viewBox", (0, 0, c.width, c.height))
        .add(path);

    svg::save("examples/image.svg", &document)?;

    Ok(())
}

fn main() -> io::Result<()> {
    let svg_config = SvgConfig::default();
    save_svg(&svg_config)?;
    Ok(())
}
