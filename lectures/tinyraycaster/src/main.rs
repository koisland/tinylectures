use std::{f32::consts::PI, path::PathBuf};

use crate::{enemy::Enemy, map::Map, player::Player, screen::Screen};

mod color;
mod enemy;
mod entity;
mod map;
mod player;
mod screen;

fn main() -> eyre::Result<()> {
    // Parse map.
    let mut map = Map::new()
        .with_map("lectures/tinyraycaster/data/map.txt")?
        .with_textures("lectures/tinyraycaster/data/tilemap_textures.tsv", 64)?
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

    for (x, y, _texture_id) in [(1.834, 8.765, 0), (5.323, 5.365, 1), (4.123, 10.265, 1)] {
        map.spawn_entity(Enemy {
            x,
            y,
            _angle: 0.,
            _texture_id,
        });
    }

    let outdir = PathBuf::from(".");
    for i in 0..360 {
        let fname = outdir.join(format!("{i}.ppm"));
        player.ang += 2.0 * PI / 360.0;
        // Render frame
        screen.render(&player, &map)?;
        // Before dumping to outfile.
        screen.dump(fname)?;
    }

    Ok(())
}
