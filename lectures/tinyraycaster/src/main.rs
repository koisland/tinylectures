use std::{error::Error, fs::File, io::Write};

use crate::map::Map;

mod map;

/// Pack color as 4-byte u32.
fn pack_color(r: u8, g: u8, b: u8, a: Option<u8>) -> u32 {
    let a: u8 = a.unwrap_or(255);
    // rgba is stored in a u32
    ((a as u32) << 24) + ((b as u32) << 16) + ((g as u32) << 8) + (r as u32)
}

fn unpack_color(color: &u32, r: &mut u8, g: &mut u8, b: &mut u8, a: &mut u8) {
    // Modify the r channel.
    // Extract the color at each channel by:
    // * Right-shift bits by 1 byte chunks to get color channel as u8.
    // * Update rgb values.
    *r = *color as u8;
    *g = (*color >> 8) as u8;
    *b = (*color >> 16) as u8;
    *a = (*color >> 24) as u8;
}

#[allow(clippy::too_many_arguments)]
fn draw_rectangle(
    image: &mut [u32],
    image_w: usize,
    image_h: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    color: u32,
) {
    assert!(
        image.len() == image_w * image_h,
        "incorrect image dimensions"
    );
    // Loop thru length and width adding px by px.
    for i in 0..w {
        for j in 0..h {
            let cx = x + i;
            let cy = y + j;
            // eprintln!("({cx}, {cy})");
            assert!(cx < image_w && cy < image_h, "out of bounds");
            image[cx + cy * image_w] = color;
        }
    }
}

/// Write PPM file.
/// https://netpbm.sourceforge.net/doc/ppm.html
fn drop_ppm_image(fname: &str, image: &[u32], w: usize, h: usize) -> Result<(), Box<dyn Error>> {
    // Check images is correct size as given width and height.
    assert_eq!(image.len(), w * h);
    let mut fh = File::create(fname)?;
    // Write magic number identifying file type, w, h, max color value. All delimited by newline.
    let ppm_mdata = format!("P3\n{w} {h}\n255\n").into_bytes();
    fh.write_all(&ppm_mdata)?;
    const END_CHAR: [&str; 2] = ["\n", " "];

    for (i, px) in image.iter().take(h * w).enumerate() {
        let (mut r, mut g, mut b, mut a) = (0, 0, 0, 0);
        // Update rgba
        unpack_color(px, &mut r, &mut g, &mut b, &mut a);
        // Place end char so after each rgb triplet, properly spaced.
        let end_char = END_CHAR[usize::from(i % w != 0)];
        let px_rgb = format!("{r} {g} {b}{end_char}").into_bytes();

        fh.write_all(&px_rgb)?;
    }

    Ok(())
}

fn main() {
    const WIDTH: usize = 512;
    const HEIGHT: usize = 512;

    let map = Map::new("lectures/tinyraycaster/data/map.txt");

    // Store image in 1D array.
    // Access elems by specify w + (h * WIDTH)
    let mut framebuffer: Vec<u32> = vec![255; WIDTH * HEIGHT];

    // Iterate through window pixels and fill with color gradient.
    for h in 0..HEIGHT {
        for w in 0..WIDTH {
            // Vary the red channel between 0-255 as h sweeps vertical.
            let r: u8 = (255 * h / HEIGHT) as u8;
            // Vary the green channel between 0-255 as w sweeps horizontal.
            let g: u8 = (255 * w / WIDTH) as u8;
            let b: u8 = 0;
            // Access index of one-dim array.
            framebuffer[w + h * WIDTH] = pack_color(r, g, b, None)
        }
    }

    // Then add map
    let rect_w = WIDTH / map.w;
    let rect_h = HEIGHT / map.h;
    eprintln!("Rects (w: {rect_w}, h: {rect_h})");
    for (x, y, tile) in map.tiles() {
        if tile.is_some_and(|t| t != " ") {
            // Because each rect is w and h
            let rect_x = x * rect_w;
            let rect_y = y * rect_h;
            eprintln!("At ({x},{y}) draw {tile:?} tile at ({rect_x}, {rect_y}) ");
            draw_rectangle(
                &mut framebuffer,
                WIDTH,
                HEIGHT,
                rect_x,
                rect_y,
                rect_w,
                rect_h,
                pack_color(0, 255, 255, None),
            );
        }
    }

    drop_ppm_image("./out.ppm", &framebuffer, WIDTH, HEIGHT).unwrap();
}
