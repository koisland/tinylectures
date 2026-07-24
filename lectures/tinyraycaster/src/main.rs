use std::{f32::consts::PI, path::PathBuf};

use crate::{
    enemy::{Enemy, EnemyState, EnemyType},
    map::Map,
    player::Player,
    screen::Screen,
};

mod color;
mod enemy;
mod map;
mod player;
mod screen;
mod tiles;

fn main() -> eyre::Result<()> {
    // Parse map.
    let mut map = Map::new()
        .with_map("lectures/tinyraycaster/data/map.txt")?
        .with_textures("lectures/tinyraycaster/data/textures.tsv", 64)?
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

    for (x, y, typ, state) in [
        (1.834, 8.765, EnemyType::RedBlob, EnemyState::Base),
        (5.323, 5.365, EnemyType::RedBlob, EnemyState::Injured),
        (4.123, 10.265, EnemyType::RedBlob, EnemyState::Injured),
    ] {
        // https://www.youtube.com/watch?v=VMYk9fqXz_4
        // https://stackoverflow.com/questions/283406/what-is-the-difference-between-atan-and-atan2-in-c
        // Use atan2 incase where x is negative. Allows getting angle with range across all 4 quadrants as opposed to 2 (1 and 4).
        let dst_y = y - player.y;
        let dst_x = x - player.x;
        // Angle of enemy relative to player
        let angle = dst_y.atan2(dst_x);

        map.spawn_enemy(Enemy {
            x,
            y,
            angle,
            state,
            typ,
        });
    }

    let outdir = PathBuf::from(".");
    for i in 0..1 {
        let fname = outdir.join(format!("{i}.ppm"));
        player.ang += 2.0 * PI / 360.0;
        // Render frame
        screen.render(&player, &map)?;
        // Before dumping to outfile.
        screen.dump(fname)?;
    }

    Ok(())
}
