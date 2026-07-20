use std::f32::consts::PI;

use crate::{map::Map, player::Player, screen::Screen};

mod color;
mod map;
mod player;
mod screen;

fn main() -> eyre::Result<()> {
    const FNAME: &str = "./out.ppm";

    // Parse map.
    let map = Map::new()
        .with_map("lectures/tinyraycaster/data/map.txt")?
        .with_textures("lectures/tinyraycaster/data/tilemap.tsv")?
        .finish()?;

    // With initialization function.
    let mut screen = Screen::<1024, 512>::new();
    // TODO: Allow setting fov in degrees but internally use radians.
    let player = Player {
        x: 3.456,
        y: 2.345,
        ang: 1.523,
        fov: PI / 3.0,
    };

    // Then add map and player.
    screen.draw_map(&map)?;
    screen.draw_player(&player, &map)?;
    // Draw fov for player
    screen.draw_fov(&player, &map)?;

    // Before dumping to outfile.
    screen.dump(FNAME)
}
