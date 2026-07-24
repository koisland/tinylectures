use std::{
    f32::consts::PI,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use eyre::bail;
use image::{DynamicImage, GenericImageView};

use crate::{
    color::Color,
    enemy::Enemy,
    map::Map,
    player::Player,
    textures::{Texture, Textures},
};

pub struct RayHit {
    // x coord hit by ray
    cx: f32,
    // y coord hit by ray
    cy: f32,
    // Distance from player to hit tile
    dst: f32,
}

// Store image in 1D array.
// Access elems by specify w + (h * WIDTH)
pub struct Screen<const W: usize, const H: usize> {
    buffer: Vec<Color>,
    depth_buffer: Vec<f32>,
}

impl<const W: usize, const H: usize> Screen<W, H> {
    pub fn new() -> Self {
        let buffer = vec![Color::new(255, 255, 255, None); W * H];
        let depth_buffer = vec![1e3; W / 2];
        Screen::<W, H> {
            buffer,
            depth_buffer,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = vec![Color::new(255, 255, 255, None); W * H];
    }

    /// Write PPM file.
    /// https://netpbm.sourceforge.net/doc/ppm.html
    pub fn dump(&self, fname: impl AsRef<Path>) -> eyre::Result<()> {
        // Check images is correct size as given width and height.
        let mut fh = BufWriter::new(File::create(fname)?);
        // Write magic number identifying file type, w, h, max color value. All delimited by newline.
        write!(fh, "P3\n{W} {H}\n255\n")?;

        const END_CHAR: [&str; 2] = ["\n", " "];
        for (i, px) in self.buffer.iter().take(H * W).enumerate() {
            let (r, g, b, _) = px.channels();
            // Place end char so after each rgb triplet, properly spaced.
            let end_char = END_CHAR[usize::from(i % W != 0)];
            write!(fh, "{r} {g} {b}{end_char}")?;
        }
        Ok(())
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.buffer[x + y * W] = color
    }

    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: Color) {
        // Loop thru length and width adding px by px.
        for i in 0..w {
            for j in 0..h {
                let cx = x + i;
                let cy = y + j;
                // eprintln!("({cx}, {cy})");
                if cx >= W || cy >= H {
                    continue;
                }
                self.draw_pixel(cx, cy, color);
            }
        }
    }

    pub fn draw_image(&mut self, x: usize, y: usize, image: &DynamicImage) {
        let (w, h) = image.dimensions();
        let (w, h) = (w as usize, h as usize);
        for i in 0..w {
            for j in 0..h {
                let cx = x + i;
                let cy = y + j;
                if cx >= W || cy >= H {
                    continue;
                }
                let [r, g, b, a] = image.get_pixel(i as u32, j as u32).0;
                self.draw_pixel(cx, cy, Color::new(r, g, b, Some(a)));
            }
        }
    }

    // TODO: Maybe move to Map.
    pub fn draw_map(
        &mut self,
        player: &Player,
        map: &Map,
        textures: &Textures,
    ) -> eyre::Result<()> {
        let rect_w = W / (map.w * 2);
        let rect_h = H / map.h;
        // eprintln!("Rects (w: {rect_w}, h: {rect_h})");

        for (x, y, tile) in map.tiles() {
            if let Some(tile) = tile {
                // Because each rect is w and h
                let rect_x = x * rect_w;
                let rect_y = y * rect_h;
                let Some(texture) = textures.get_tile(tile) else {
                    bail!("Tile {tile:?} has no texture.")
                };

                // eprintln!("At ({x},{y}) draw {tile:?} tile at ({rect_x}, {rect_y}) ");
                match texture {
                    Texture::Color(color) => {
                        self.draw_rect(rect_x, rect_y, rect_w, rect_h, *color);
                    }
                    Texture::Sprite(img) => {
                        // Draw thumbnail
                        let img_thumbnail = img.thumbnail(rect_w as u32, rect_h as u32);
                        self.draw_image(rect_x, rect_y, &img_thumbnail);
                    }
                };
            }
            continue;
        }
        self.draw_player_on_map(player, map);
        self.draw_entities_on_map(map);
        Ok(())
    }

    // TODO: Refactor draw_* to take a struct that implents and Entity trait
    pub fn draw_player_on_map(&mut self, player: &Player, map: &Map) {
        let rect_w = W / (map.w * 2);
        let rect_h = H / map.h;
        // Convert from coordinates to image dim
        let x = (player.x * rect_w as f32) as usize;
        let y = (player.y * rect_h as f32) as usize;
        self.draw_rect(x, y, 5, 5, Color::new(0, 0, 0, None));
    }

    pub fn draw_entities_on_map(&mut self, map: &Map) {
        let rect_w = W / (map.w * 2);
        let rect_h = H / map.h;

        for entity in map.id_enemy_map.values() {
            let x = (entity.x * rect_w as f32) as usize;
            let y = (entity.y * rect_h as f32) as usize;
            self.draw_rect(x, y, 5, 5, Color::new(255, 0, 0, None));
        }
    }

    pub fn draw_sprite(
        &mut self,
        enemy: &Enemy,
        player: &Player,
        textures: &Textures,
    ) -> eyre::Result<()> {
        // https://www.youtube.com/watch?v=VMYk9fqXz_4
        // https://stackoverflow.com/questions/283406/what-is-the-difference-between-atan-and-atan2-in-c
        // Use atan2 incase where x is negative. Allows getting angle with range across all 4 quadrants as opposed to 2 (1 and 4).
        let dst_y = enemy.y - player.y;
        let dst_x = enemy.x - player.x;
        // Angle of enemy relative to player
        let mut angle = dst_y.atan2(dst_x);
        while angle - player.ang > PI {
            angle -= 2. * PI;
        } // remove unncesessary periods from the relative direction
        while angle - player.ang < -PI {
            angle += 2. * PI;
        }

        // pythagorean theorem
        let sprite_dst = ((player.x - enemy.x).powi(2) + (player.y - enemy.y).powi(2)).sqrt();
        let Some(texture) = textures.get_enemy(enemy) else {
            bail!("No texture for enemy {enemy:?}")
        };
        // Scale sprite by distance from player and clamp to 2000 if very close.
        let sprite_screen_size = ((H as f32 / sprite_dst) as usize).min(1000);
        let hscreen_width = W / 2;
        let hscreen_height = H / 2;
        let hsprite_screen_size = sprite_screen_size / 2;

        // Get the upper left corner of the sprite
        // TODO: Huh? Unclear why this works.
        let h_offset_pt1 = (angle - player.ang) / player.fov * hscreen_width as f32;
        let h_offset_pt2 = (hscreen_width / 2 - textures.size / 2) as f32;
        let h_offset = (h_offset_pt1 + h_offset_pt2) as usize;
        let v_offset = hscreen_height - hsprite_screen_size;

        for i in 0..sprite_screen_size {
            let px_x = h_offset + i;
            // Don't draw horizontal pixel if OOB
            if px_x >= hscreen_width {
                continue;
            }
            // Occluded. Pixel at x-pos in front of sprite (closer).
            if self.depth_buffer[px_x] < sprite_dst {
                continue;
            }

            for j in 0..sprite_screen_size {
                let px_y = v_offset + j;
                if px_y >= H {
                    continue;
                }
                let Some(color) = texture.get_color(
                    i * textures.size / sprite_screen_size,
                    j * textures.size / sprite_screen_size,
                ) else {
                    bail!("No color.")
                };

                let (_, _, _, a) = color.channels();
                // Only draw opaque pixels
                // https://colorlabs.net/posts/what-are-alpha-channels-in-digital-images
                if a > 128 {
                    self.draw_pixel(hscreen_width + px_x, px_y, color);
                }
            }
        }
        Ok(())
    }

    pub fn draw_sprites(
        &mut self,
        player: &Player,
        map: &Map,
        textures: &Textures,
    ) -> eyre::Result<()> {
        for entity in map.id_enemy_map.values() {
            self.draw_sprite(entity, player, textures)?;
        }
        Ok(())
    }

    /// # Drawing a ray.
    /// Our diagram of the player in space looks like this.
    /// ```no_run
    ///  a
    /// ___
    /// \p |
    ///  \ | b
    /// c \|
    ///    (x, y)
    /// ```
    ///
    /// Remember soh-cah-toa? This allows us to calculate `x` and `y` from `p_angle` (`ang`).
    /// * `cos(p_angle) = a/c` which also is `a = c * cos(p_angle)`
    /// * `sin(p_angle) = b/c` which also is `b = c * cos(p_angle)`
    ///
    /// So:
    /// * `x` and `y` is the endpoint of the ray (hypotenuse of c) along the triangle.
    /// * `c` is some arbitrary value representing the distance from object hit by ray
    ///
    /// Thus:
    /// * `x = p_x + c * cos(p_angle)`
    /// * `y = p_y + c * sin(p_angle)`
    ///
    /// This function returns the distance (length of c) to the endpoint of the ray.
    pub fn draw_ray(
        &mut self,
        x: f32,
        y: f32,
        ang: f32,
        map: &Map,
        textures: &Textures,
        mut f_hit: impl FnMut(&mut Screen<W, H>, &Texture, RayHit),
    ) -> eyre::Result<f32> {
        let rect_w = (W / (map.w * 2)) as f32;
        let rect_h = (H / map.h) as f32;

        // We don't include a limit (20) unlike the src
        const INC: f32 = 0.01;
        let mut dst: f32 = 0.0;
        loop {
            let cx = x + dst * ang.cos();
            let cy = y + dst * ang.sin();

            // Otherwise, draw ray
            let px_x = cx * rect_w;
            let px_y = cy * rect_h;
            self.draw_rect(
                px_x as usize,
                px_y as usize,
                1,
                1,
                Color::new(160, 160, 160, None),
            );

            // Out of bounds or hit an object
            if let Some(htile) = map.tile(cx as usize, cy as usize) {
                // Call function on hit.
                // TODO: Abstract to function to handle cases where no key
                let Some(texture) = &textures.get_tile(htile) else {
                    bail!("No texture for hit tile {htile:?}")
                };
                f_hit(self, texture, RayHit { cx, cy, dst });
                break;
            };

            dst += INC
        }
        Ok(dst)
    }

    /// # Generate the field-of-view of the player
    /// ```no_run
    ///    ------
    ///   /2\2 \1
    /// f/   \d \
    /// ```
    ///
    /// * 1 is the angle between x-axis and fov angle
    /// * 2 is the fov angle centered on the player's direction angle
    /// * Add both together to calculate the fov
    ///
    /// We iterate over the width because it is the hypotenuse of the FOV tri/cone.
    ///
    /// # To adjust for fisheye distortion:
    /// See https://gamedev.stackexchange.com/a/97580 for diagram.
    /// * Because the height of the walls are determined based on distance, distant rays in the fov are longer and create shorter walls.
    /// * We need to take the range instead of the distance to determine wall height.
    ///
    /// # Drawing textures
    /// In order to draw the texture, we have to know where on the tile was hit.
    /// * It could be on the horizontal (hitx) or vertical (hity)
    /// * They contain (signed) fractional parts of cx and cy (endpoint coordinates of the ray) from 0.5 to -0.5
    /// * The large magnitude indicates that it is the one hit. We can get the coordinate in sprite space as a result.
    /// * Then we draw it.
    pub fn draw_fov(
        &mut self,
        player: &Player,
        map: &Map,
        textures: &Textures,
    ) -> eyre::Result<()> {
        let fw: f32 = (W / 2) as f32;
        // Angle between x-axis and fov
        // Direction - (FOV / 2)
        let pt_1 = player.ang - player.fov / 2.;
        for i in 0..(W / 2) {
            // The rest of the FOV angle drawn section by section.
            // (FOV * 0..512) / 512.
            let pt_2 = player.fov * (i as f32 / fw);
            let angle = pt_1 + pt_2;
            self.draw_ray(
                player.x,
                player.y,
                angle,
                map,
                textures,
                move |img, tile, ray_hit| {
                    // Closer means smaller c and thus large ht.
                    // We need to adjust this scaling to avoid fisheye distortion due to the ray hitting at multiple angles
                    // See https://gamedev.stackexchange.com/a/97580
                    // And https://lodev.org/cgtutor/raycasting.html
                    let col_ht = (H as f32 / (ray_hit.dst * (angle - player.ang).cos())) as usize;
                    // Draw at every angle within FOV
                    let col_x = W / 2 + i;

                    // Store distance to know what to occlude from fov
                    img.depth_buffer[i] = ray_hit.dst;

                    // Draw texture/tile
                    match tile {
                        Texture::Color(color) => {
                            // Start at middle of screen and then drop y by half the col ht. This centers the drawn line.
                            let col_y = H / 2 - col_ht / 2;
                            img.draw_rect(col_x, col_y, 1, col_ht, *color);
                        }
                        Texture::Sprite(sprite) => {
                            let size = sprite.height() as f32;
                            // We need to know whether we hit the x or y side of the texture.
                            // hitx and hity contain (signed) fractional parts of cx and cy from 0.5 to -0.5
                            // If hity (fractional part of y) magnitude larger, then "vertical" part of tile hit.
                            //  hitx             hity
                            //  *
                            // ______              ______
                            // |    |            * |    |
                            // |____|              |____|
                            let hitx = ray_hit.cx - (ray_hit.cx + 0.5).floor();
                            let hity = ray_hit.cy - (ray_hit.cy + 0.5).floor();
                            // Once know part of texture was hit, we can get what part of sprite to render from the size and fraction.
                            let mut x_texcoord = if hity.abs() > hitx.abs() {
                                hity * size
                            } else {
                                hitx * size
                            };
                            if x_texcoord < 0.0 {
                                x_texcoord += size
                            }
                            assert!(x_texcoord >= 0.0 && x_texcoord < size);

                            // Scale column to height.
                            let mut texcol = vec![];
                            for y in 0..col_ht {
                                let pix_y = (y as f32 * size) / col_ht as f32;
                                // eprintln!("{y}*{size}/{col_ht}={pix_y}");
                                texcol.push(sprite.get_pixel(x_texcoord as u32, pix_y as u32));
                            }
                            // Write scaled column
                            for (j, [r, g, b, a]) in
                                texcol.iter().map(|px| px.0).enumerate().take(col_ht)
                            {
                                // Start at middle of screen ht, then half of the column ht. Add pixels to reach col_ht again.
                                let pix_y = j + (H / 2) - (col_ht / 2);
                                if pix_y >= H {
                                    continue;
                                }
                                img.draw_pixel(col_x, pix_y, Color::new(r, g, b, Some(a)));
                            }
                        }
                    };
                },
            )?;
        }
        Ok(())
    }

    pub fn render(&mut self, player: &Player, map: &Map, textures: &Textures) -> eyre::Result<()> {
        // Clear buffer
        self.clear();
        // Draw fov for player
        self.draw_fov(player, map, textures)?;
        // Then draw map and player.
        self.draw_map(player, map, textures)?;
        // And draw sprites
        self.draw_sprites(player, map, textures)?;
        Ok(())
    }
}
