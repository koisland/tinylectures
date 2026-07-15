use std::{error::Error, fs::File, io::Write};

/// Pack color as 4-byte u32.
fn pack_color(r: u8, g: u8, b: u8, a: Option<u8>) -> u32 {
    let a: u8 = a.unwrap_or(255);
    // rgba is stored in a u32
    ((a as u32) << 24) +((b as u32) << 16) + ((g as u32) << 8) + (r as u32) 
}

fn unpack_color(color: &u32, r: &mut u8, g: &mut u8, b: &mut u8, a: &mut u8) {
    // Modify the r channel.
    // Extract the color at each channel by:
    // * Right-shift bits by 1 byte chunks to get color channel as u8.
    // * Update rgb values.
    *r = *color as u8 & 255;
    *g = (*color >> 8) as u8 & 255;
    *b = (*color >> 16) as u8 & 255;
    *a = (*color >> 24) as u8 & 255;
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
    drop_ppm_image("./out.ppm", &framebuffer, WIDTH, HEIGHT).unwrap();
}
