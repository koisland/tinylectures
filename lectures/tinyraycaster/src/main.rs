use std::f32::consts::PI;

use crate::{image::Image, map::Map, player::Player};

mod color;
mod image;
mod map;
mod player;

fn main() -> eyre::Result<()> {
    const FNAME: &str = "./out.ppm";

    // Parse map.
    let map = Map::new()
        .with_map("lectures/tinyraycaster/data/map.txt")?
        .with_textures("lectures/tinyraycaster/data/tilemap.tsv")?
        .finish();

    // With initialization function.
    let mut image = Image::<1024, 512>::new();
    // TODO: Allow setting fov in degrees but internally use radians.
    let player = Player {
        x: 3.456,
        y: 2.345,
        ang: 1.523,
        fov: PI / 3.0,
    };

    // Then add map and player.
    image.draw_map(&map)?;
    image.draw_player(&player, &map)?;
    // Draw fov for player
    image.draw_fov(&player, &map)?;

    // Before dumping to outfile.
    image.dump(FNAME)
}
