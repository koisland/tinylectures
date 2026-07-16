use std::{error::Error, fs::File, io::Write};

use crate::map::Map;

mod map;

/// RGBA color as 4-byte u32.
#[derive(Debug, Clone, Copy)]
pub struct Color(u32);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: Option<u8>) -> Self {
        let a: u8 = a.unwrap_or(255);
        // rgba is stored in a u32
        Self(((a as u32) << 24) + ((b as u32) << 16) + ((g as u32) << 8) + (r as u32))
    }

    pub fn channels(&self) -> (u8, u8, u8, u8) {
        // Modify the r channel.
        // Extract the color at each channel by:
        // Right-shift bits by 1 byte chunks to get color channel as u8.
        (
            self.0 as u8,
            (self.0 >> 8) as u8,
            (self.0 >> 16) as u8,
            (self.0 >> 24) as u8,
        )
    } 
}

// Store image in 1D array.
// Access elems by specify w + (h * WIDTH)
pub struct Image<const W: usize, const H: usize> {
    buffer: Vec<Color>
}

impl<const W: usize, const H: usize> Image<W, H> {
    pub fn new(f: impl Fn(usize, usize) -> (u8, u8, u8)) -> Self {
        let mut buffer = vec![Color(255); W * H];
        // Iterate through window pixels and fill with color gradient.
        for h in 0..H {
            for w in 0..W {
                // Vary the red channel between 0-255 as h sweeps vertical.
                let (r, g, b) = f(h, w);
                // Access index of one-dim array.
                buffer[w + h * W] = Color::new(r, g, b, None)
            }
        }
        Image::<W, H> { buffer }
    }
    
    /// Write PPM file.
    /// https://netpbm.sourceforge.net/doc/ppm.html
    pub fn dump(&self, fname: &str) -> Result<(), Box<dyn Error>> {
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
                assert!(cx < W && cy < H, "out of bounds");
                self.buffer[cx + cy * W] = color;
            }
        }
    }

    pub fn draw_map(&mut self, map: &Map) {
        let rect_w = W / map.w;
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
    }
}

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
