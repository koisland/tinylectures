use std::{f32::consts::PI, path::PathBuf};

use crate::{map::Map, player::Player, screen::Screen};

mod color;
mod map;
mod player;
mod screen;

fn main() -> eyre::Result<()> {
    // Parse map.
    let map = Map::new()
        .with_map("lectures/tinyraycaster/data/map.txt")?
        .with_textures("lectures/tinyraycaster/data/tilemap.tsv")?
        .finish()?;

    // With initialization function.
    let mut screen = Screen::<1024, 512>::new();
    // TODO: Allow setting fov in degrees but internally use radians.
    let mut player = Player {
        x: 3.456,
        y: 2.345,
        ang: 1.523,
        fov: PI / 3.0,
    };

    let outdir = PathBuf::from(".");
    for i in 0..360 {
        let fname = outdir.join(format!("{i}.ppm"));
        player.ang += 2.0 * PI / 360.0;

        screen.clear();
        // Then draw map and player again.
        // Draw fov for player
        screen.draw_map(&map)?;
        screen.draw_player(&player, &map)?;
        screen.draw_fov(&player, &map)?;
        // Before dumping to outfile.
        screen.dump(fname)?;
    }

    Ok(())
}
