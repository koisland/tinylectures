use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug, Clone)]
pub struct Map {
    pub(crate) src: String,
    pub(crate) w: usize,
    pub(crate) h: usize,
}

impl Map {
    pub fn new(infile: &str) -> Self {
        let fh = BufReader::new(File::open(infile).unwrap());
        let mut map = String::new();
        let mut map_w: usize = 0;
        let mut map_h: usize = 0;
        for (h, line) in fh.lines().enumerate() {
            let line = line.unwrap();
            let line = line.trim();
            let w = line.len();

            map.push_str(line);
            // eprintln!("{line} ({w}, {h})");
            // Only check at end. Could also do here and give failing line
            map_w = w;
            // 0-index
            map_h = h + 1;
        }

        assert_eq!(
            map_w * map_h,
            map.len(),
            "Map does not have uniform length and width"
        );
        Map {
            src: map,
            w: map_w,
            h: map_h,
        }
    }

    #[inline]
    pub fn tile(&self, x: usize, y: usize) -> Option<&str> {
        let idx = x + y * self.w;
        self.src.get(idx..idx + 1)
    }

    pub fn tiles(&self) -> impl Iterator<Item = (usize, usize, Option<&str>)> {
        (0..self.h).flat_map(move |y| (0..self.w).map(move |x| (x, y, self.tile(x, y))))
    }
}
