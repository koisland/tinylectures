use std::{fs::File, io::Write};

use crate::{color::Color, map::Map, player::Player};

// Store image in 1D array.
// Access elems by specify w + (h * WIDTH)
pub struct Image<const W: usize, const H: usize> {
    buffer: Vec<Color>,
}

impl<const W: usize, const H: usize> Image<W, H> {
    pub fn new() -> Self {
        let buffer = vec![Color::new(255, 255, 255, None); W * H];
        Image::<W, H> { buffer }
    }

    /// Write PPM file.
    /// https://netpbm.sourceforge.net/doc/ppm.html
    pub fn dump(&self, fname: &str) -> eyre::Result<()> {
        // Check images is correct size as given width and height.
        let mut fh = File::create(fname)?;
        // Write magic number identifying file type, w, h, max color value. All delimited by newline.
        let ppm_mdata = format!("P3\n{W} {H}\n255\n").into_bytes();
        fh.write_all(&ppm_mdata)?;
        const END_CHAR: [&str; 2] = ["\n", " "];

        for (i, px) in self.buffer.iter().take(H * W).enumerate() {
            let (r, g, b, _) = px.channels();
            // Place end char so after each rgb triplet, properly spaced.
            let end_char = END_CHAR[usize::from(i % W != 0)];
            let px_rgb = format!("{r} {g} {b}{end_char}").into_bytes();

            fh.write_all(&px_rgb)?;
        }
        Ok(())
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
                self.buffer[cx + cy * W] = color;
            }
        }
    }

    // TODO: Maybe move to Map.
    pub fn draw_map(&mut self, map: &Map) -> eyre::Result<()> {
        let rect_w = W / (map.w * 2);
        let rect_h = H / map.h;
        eprintln!("Rects (w: {rect_w}, h: {rect_h})");

        let color = Color::new(0, 255, 255, None);
        for (x, y, tile) in map.tiles() {
            if tile.is_some_and(|t| t != " ") {
                // Because each rect is w and h
                let rect_x = x * rect_w;
                let rect_y = y * rect_h;
                eprintln!("At ({x},{y}) draw {tile:?} tile at ({rect_x}, {rect_y}) ");
                self.draw_rect(rect_x, rect_y, rect_w, rect_h, color);
            }
        }
        Ok(())
    }

    // TODO: Refactor draw_* to take a struct that implents and Entity trait
    pub fn draw_player(&mut self, player: &Player, map: &Map) -> eyre::Result<()> {
        let rect_w = W / (map.w * 2);
        let rect_h = H / map.h;
        // Convert from coordinates to image dim
        let x = (player.x * rect_w as f32) as usize;
        let y = (player.y * rect_h as f32) as usize;
        self.draw_rect(x, y, 5, 5, Color::new(255, 255, 255, None));
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
        mut f_hit: impl FnMut(&mut Image<W, H>, f32),
    ) -> eyre::Result<f32> {
        let rect_w = (W / (map.w * 2)) as f32;
        let rect_h = (H / map.h) as f32;

        // We don't include a limit (20) unlike the src
        const INC: f32 = 0.05;
        let mut c: f32 = 0.0;
        loop {
            let cx = x + c * ang.cos();
            let cy = y + c * ang.sin();

            // Otherwise, draw ray
            let px_x = (cx * rect_w) as usize;
            let px_y = (cy * rect_h) as usize;
            self.draw_rect(px_x, px_y, 1, 1, Color::new(160, 160, 160, None));

            // Out of bounds or hit an object
            if map.tile(cx as usize, cy as usize).is_none_or(|t| t != " ") {
                // Call function on hit.
                f_hit(self, c);
                break;
            };

            c += INC
        }
        Ok(c)
    }

    /// Generate the field-of-view of the player
    ///
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
    pub fn draw_fov(&mut self, player: &Player, map: &Map) -> eyre::Result<()> {
        let fw: f32 = (W / 2) as f32;
        // Angle between x-axis and fov
        // Direction - (FOV / 2)
        let pt_1 = player.ang - player.fov / 2.;
        for i in 0..(W / 2) {
            // The rest of the FOV angle drawn section by section.
            // (FOV * 0..512) / 512.
            let pt_2 = player.fov * (i as f32 / fw);
            let angle = pt_1 + pt_2;
            self.draw_ray(player.x, player.y, angle, map, move |img, c| {
                // Closer means smaller c and thus large ht.
                let col_ht = (H as f32 / c) as usize;
                // Draw at every angle within FOV
                let col_x = W / 2 + i;
                // Start at middle of screen and then drop y by half the col ht. This centers the drawn line.
                let col_y = H / 2 - col_ht / 2;
                img.draw_rect(col_x, col_y, 1, col_ht, Color::new(0, 255, 255, None));
            })?;
        }
        Ok(())
    }
}
