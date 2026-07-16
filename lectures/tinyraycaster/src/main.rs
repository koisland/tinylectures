use crate::{image::Image, map::Map};

mod color;
mod image;
mod map;

fn main() {
    const WIDTH: usize = 512;
    const HEIGHT: usize = 512;
    const FNAME: &str = "./out.ppm";

    // Parse map.
    let map = Map::new("lectures/tinyraycaster/data/map.txt");
    // With initialization function.
    let mut image = Image::<WIDTH, HEIGHT>::new(|h, w| {
        let r: u8 = (255 * h / HEIGHT) as u8;
        // Vary the green channel between 0-255 as w sweeps horizontal.
        let g: u8 = (255 * w / WIDTH) as u8;
        let b: u8 = 0;
        (r, g, b)
    });

    // Then add map.
    image.draw_map(&map);
    // Before dumping to outfile.
    image.dump(FNAME).unwrap();
}
