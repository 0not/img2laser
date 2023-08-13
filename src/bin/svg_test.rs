use svg::node::element::path::{Command, Data, Position};
use svg::node::element::Path;
use svg::Document;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = 100;
    let height = 100;
    let fs: f32 = 10.; // spatial sample frequency
    let start_x = 0.;
    let start_y = 2.;
    let amp = 1.8;

    // Create sine wave
    let sine1 = (0..width * (fs as i32))
        .flat_map(|xi| {
            let x = start_x + (xi as f32) / fs;
            let y = start_y + amp * (x).sin();
            vec![x, y]
        })
        .collect::<Vec<f32>>();

    let sine2 = (0..width * (fs as i32))
        .flat_map(|xi| {
            let x = start_x + (xi as f32) / fs;
            let y = 6. + amp * x.sin();
            vec![x, y]
        })
        .collect::<Vec<f32>>();

    let data = Data::new()
        .move_to((0, 2))
        .add(Command::Line(Position::Absolute, sine1.into()))
        .move_to((0, 6))
        .add(Command::Line(Position::Absolute, sine2.into()));

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 0.1)
        .set("d", data);

    let document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .add(path);

    svg::save("examples/image.svg", &document)?;

    Ok(())
}

// use plotters::prelude::*;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let drawing_area = SVGBackend::new("examples/chart_builder_on.svg", (300, 200))
//         .into_drawing_area();

//     drawing_area.draw(&PathElement::new(
//         (0..500).map(|x| (x / 10, 10 + (10.*((x / 10) as f32).sin()) as i32)).collect::<Vec<_>>()
//         , &BLACK))?;
//     // chart
//         // .draw_series(LineSeries::new(
//         //     (-500..=500).map(|x| x as f32 / 500.0).map(|x| (x, x * x)),
//         //     &RED,
//         // ))?
//         // .draw_series(std::iter::once(&PathElement::new(vec![(-1, 0), (1, 0)], &RED)));
//         // .label("y = x^2")
//         // .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

//     // chart_context
//     //     .configure_mesh()
//     //     .draw()
//     //     .unwrap();

//     Ok(())
// }
