use crate::{image::Image, map::Map, player::Player};

mod color;
mod image;
mod map;
mod player;

fn main() {
    const FNAME: &str = "./out.ppm";

    // Parse map.
    let map = Map::new("lectures/tinyraycaster/data/map.txt");
    // With initialization function.
    let mut image = Image::<512, 512>::new(|_h, _w| (0, 0, 0));
    let player = Player {
        x: 3.456,
        y: 2.345,
        a: 1.523,
    };

    // Then add map and player.
    image.draw_map(&map);
    image.draw_player(&player, &map);
    // Draw ray for player perspective
    image.draw_ray(player.x, player.y, player.a, &map);

    // Before dumping to outfile.
    image.dump(FNAME).unwrap();
}
